use anyhow::Result;
use image::{ImageBuffer, Rgb};
use priority_queue::PriorityQueue;
use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet, VecDeque, hash_map::Entry},
};
use thiserror::Error;

const STRAIGHT_COST: i32 = 1;
const TURN_COST: i32 = 1001;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Invalid character: {0}")]
    InvalidCharacter(char),
    #[error("Start point missing")]
    MissingStart,
    #[error("End point missing")]
    MissingEnd,
    #[error("No path found")]
    NoPathFound,
    #[error("Point not found: {0}")]
    PointNotFound(Point),
    #[error("Backtracking failed")]
    BacktrackingFailed,
}

// Represents a 2d point in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the point that is a neighbour in the given direction.
    pub fn neighbour(&self, direction: Direction) -> Self {
        match direction {
            Direction::North => Self::new(self.x, self.y - 1),
            Direction::South => Self::new(self.x, self.y + 1),
            Direction::East => Self::new(self.x + 1, self.y),
            Direction::West => Self::new(self.x - 1, self.y),
        }
    }

    /// Returns the manhattan distance between two points.
    pub fn distance(&self, other: Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Gets the direction opposite to the current direction.
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Gets the two directions 90 degrees to the left and right.
    pub fn perpendicular(&self) -> (Self, Self) {
        match self {
            Direction::North => (Direction::West, Direction::East),
            Direction::South => (Direction::East, Direction::West),
            Direction::East => (Direction::North, Direction::South),
            Direction::West => (Direction::South, Direction::North),
        }
    }
}

/// A graph representing a 2D grid of nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    /// A map of points to node ids.
    nodes: HashMap<Point, Node>,
    width: i32,
    height: i32,
}

impl Graph {
    /// Creates a new graph from a HashSet of points.
    /// Assumes that the points are connected in a grid-like manner.
    pub fn new(pos_map: HashSet<Point>, width: i32, height: i32) -> Self {
        let mut nodes = HashMap::new();
        for point in &pos_map {
            // Calculate all the possible neighbour points.
            let north = point.neighbour(Direction::North);
            let south = point.neighbour(Direction::South);
            let east = point.neighbour(Direction::East);
            let west = point.neighbour(Direction::West);

            // Helper to map a point to Option if it is a valid point in the graph.
            let get_direction = |point: Point| -> Option<Point> { pos_map.get(&point).copied() };

            // Create a new node.
            // For each neighbour, check if it is a valid point in the graph.
            let node = Node {
                north: get_direction(north),
                south: get_direction(south),
                east: get_direction(east),
                west: get_direction(west),
            };

            nodes.insert(*point, node);
        }
        Self {
            nodes,
            width,
            height,
        }
    }

    /// Gets a node at the given point.
    /// Returns Error if the point is not part of the graph.
    pub fn get(&self, point: &Point) -> Result<Node, GraphError> {
        self.nodes
            .get(point)
            .copied()
            .ok_or(GraphError::PointNotFound(*point))
    }

    /// Uses a modiiied A* algorithm to find all shortest paths from start to end.
    /// Returns a vector of paths.
    /// Heavily inspired by the `astar_bag` function in the `pathfinding` crate.
    /// https://github.com/evenfurther/pathfinding/blob/main/src/directed/astar.rs#L173
    fn astar_all_paths(&self, start: Point, end: Point) -> Result<(Vec<Path>, i32), GraphError> {
        self.get(&start)?; // Check if the start point is valid.
        self.get(&end)?; // Check if the end point is valid.
        if start == end {
            return Ok((vec![Path::new(vec![start])], 0));
        }
        let initial_heading = Direction::East; // Always true, per the problem statement.
        let mut frontier = PriorityQueue::new();
        let mut parents = HashMap::new();
        let mut min_cost = None; // Minimum cost to reach the end point.
        frontier.push((start, initial_heading, 0), Reverse(0)); // Reversed for min heap.
        parents.insert((start, initial_heading), (None, 0)); // Parent, cost.

        while let Some(((current, heading, cost), est_cost)) = frontier.pop() {
            if matches!(min_cost, Some(min) if est_cost.0 > min) {
                break; // If the estimated cost is greater than the minimum cost, break.
            }
            let parent_cost = parents[&(current, heading)].1;
            if current == end {
                min_cost = Some(parent_cost); // Update the minimum cost.
            }
            if cost > parent_cost {
                continue; // Skip if we've explored this way at a lower cost.
            }
            let node = self.get(&current)?;
            let neighbours = node.get_neighbours(heading);

            for (next, direction, new_cost) in neighbours
                .iter()
                .filter_map(|n| n.0.map(|point| (point, n.1, n.2)))
            {
                let new_cost = parent_cost + new_cost; // New cost to reach the next point.
                let h = next.distance(end); // Heuristic cost.
                match parents.entry((next, direction)) {
                    Entry::Vacant(e) => {
                        e.insert((Some((current, direction)), new_cost));
                    }
                    Entry::Occupied(mut e) if e.get().1 > new_cost => {
                        *e.get_mut() = (Some((current, direction)), new_cost);
                    }
                    _ => continue,
                }
                frontier.push((next, direction, new_cost), Reverse(new_cost + h));
            }
        }
        let min_cost = min_cost.ok_or(GraphError::NoPathFound)?; // If no path was found, return an error.

        // BACKTRACKING
        let mut all_paths = Vec::new();
        let mut backtrace: HashMap<Point, HashMap<Option<Point>, i32>> = HashMap::new();
        for ((point, _), (parent, cost)) in parents.iter() {
            backtrace
                .entry(*point)
                .or_default() // Create a new HashMap if the point is not in the map.
                .insert(parent.map(|(p, _)| p), *cost); // Insert the parent and cost.
        }
        let mut stack = VecDeque::new();
        stack.push_back((end, (vec![end], min_cost)));
        while let Some((point, (path, cost))) = stack.pop_front() {
            if point == start {
                all_paths.push(Path::new(path));
                continue;
            }
            let ps = backtrace.get(&point).unwrap();
            if ps.is_empty() {
                return Err(GraphError::BacktrackingFailed); // Should never happen.
            }
            for (p, c) in ps.iter() {
                let p = p.unwrap();
                // 0 is a hack, it's only needed for the first step.
                if matches!(cost - c, 0 | STRAIGHT_COST | TURN_COST) {
                    let mut new_path = path.clone();
                    new_path.push(p);
                    stack.push_back((p, (new_path, *c)));
                }
            }
        }
        Ok((all_paths, min_cost))
    }

    /// Helper function that prints a graph along with every node in a path.
    /// Needs to be told the size since the graph is not stored as a 2D array.
    pub fn draw(&self, paths: &HashSet<Point>) {
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Point::new(x, y);
                if paths.contains(&point) {
                    print!("O");
                } else if self.nodes.contains_key(&point) {
                    print!("·"); // Middle Dot not a period.
                } else {
                    print!("$");
                }
            }
            println!();
        }
        println!();
    }

    /// Just for fun, draw the graph to a BMP file.
    pub fn save_bmp(&self, paths: &HashSet<Point>, filename: &str) -> Result<()> {
        let width = self.width as u32;
        let height = self.height as u32;
        let mut img = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let point = Point::new(x as i32, y as i32);
                let pixel = if paths.contains(&point) {
                    Rgb([255u8, 0, 0]) // Red for 'O'
                } else if self.nodes.contains_key(&point) {
                    if (x + y) % 2 == 0 {
                        Rgb([255u8, 255, 255]) // White for '·' on even tiles
                    } else {
                        Rgb([200u8, 200, 200]) // Light gray for '·' on odd tiles
                    }
                } else {
                    Rgb([0u8, 0, 0]) // Black for '$'
                };
                img.put_pixel(x, y, pixel);
            }
        }

        // Create the necessary directories
        if let Some(parent) = std::path::Path::new(filename).parent() {
            std::fs::create_dir_all(parent)?;
        }

        img.save(filename)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Path {
    points: Vec<Point>,
}

impl Path {
    pub fn new(points: Vec<Point>) -> Self {
        Self { points }
    }

    pub fn length(&self) -> usize {
        self.points.len()
    }

    /// Converts the path to a HashSet of points.
    pub fn to_set(&self) -> HashSet<Point> {
        self.points.iter().copied().collect()
    }
}

/// A node in the graph.
/// Contains the potential neighbours in each direction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Node {
    north: Option<Point>,
    south: Option<Point>,
    east: Option<Point>,
    west: Option<Point>,
}

impl Node {
    /// Returns the neighbours in the given direction.
    pub fn get_neighbours(&self, direction: Direction) -> [(Option<Point>, Direction, i32); 3] {
        let (left, right) = direction.perpendicular();
        [
            (self.neighbour(direction), direction, STRAIGHT_COST),
            (self.neighbour(left), left, TURN_COST),
            (self.neighbour(right), right, TURN_COST),
        ]
    }

    fn neighbour(&self, direction: Direction) -> Option<Point> {
        match direction {
            Direction::North => self.north,
            Direction::South => self.south,
            Direction::East => self.east,
            Direction::West => self.west,
        }
    }
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;
    let (graph, start, end) = parse_input(&input)?;
    // Pathfinding benchmarking.
    let start_time = std::time::Instant::now();
    let (paths, cost) = graph.astar_all_paths(start, end)?;
    let elapsed = start_time.elapsed();
    println!("Pathfinding completed in {}ms", elapsed.as_millis());

    println!("Part 1: {}", cost);

    let unique = unique_points_in_paths(&paths);
    println!("Part 2: {}", unique.len());

    // Prints the graph as a bmp file.
    graph.save_bmp(&unique, "images/output.bmp")?;

    Ok(())
}

fn unique_points_in_paths(paths: &[Path]) -> HashSet<Point> {
    paths
        .iter()
        .flat_map(|p| p.points.iter().copied())
        .collect()
}

pub fn parse_input(input: &str) -> Result<(Graph, Point, Point), GraphError> {
    // Start by normalizing line endings to \n.
    let s = input.replace("\r\n", "\n");
    let mut pos_map = HashSet::new();
    let mut start = None;
    let mut end = None;
    let mut width = 0;
    let mut height = 0;

    for (y, line) in s.trim().lines().enumerate() {
        height = y;
        for (x, c) in line.trim().char_indices() {
            width = x;
            let point = Point::new(x as i32, y as i32);
            match c {
                // Empty Space
                '.' => {
                    pos_map.insert(point);
                }
                // Start Point
                'S' => {
                    pos_map.insert(point);
                    start = Some(point);
                }
                // End Point
                'E' => {
                    pos_map.insert(point);
                    end = Some(point);
                }
                // Do nothing for walls.
                '#' => {}
                _ => return Err(GraphError::InvalidCharacter(c)),
            }
        }
    }
    // Check if start and end points are present.
    let start = start.ok_or(GraphError::MissingStart)?;
    let end = end.ok_or(GraphError::MissingEnd)?;
    Ok((
        Graph::new(pos_map, (width + 1) as i32, (height + 1) as i32),
        start,
        end,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    const INPUT_ONE: &str = r"
        ###############
        #.......#....E#
        #.#.###.#.###.#
        #.....#.#...#.#
        #.###.#####.#.#
        #.#.#.......#.#
        #.#.#####.###.#
        #...........#.#
        ###.#.#####.#.#
        #...#.....#.#.#
        #.#.#.###.#.#.#
        #.....#...#.#.#
        #.###.#.#.#.#.#
        #S..#.....#...#
        ###############
    ";

    const INPUT_TWO: &str = r"
        #################
        #...#...#...#..E#
        #.#.#.#.#.#.#.#.#
        #.#.#.#...#...#.#
        #.#.#.#.###.#.#.#
        #...#.#.#.....#.#
        #.#.#.#.#.#####.#
        #.#...#.#.#.....#
        #.#.#####.#.###.#
        #.#.#.......#...#
        #.#.###.#####.###
        #.#.#...#.....#.#
        #.#.#.#####.###.#
        #.#.#.........#.#
        #.#.#.#########.#
        #S#.............#
        #################
";

    #[test]
    fn test_parse_input() {
        let (graph, start, end) = parse_input(INPUT_ONE).unwrap();

        assert_eq!(start, Point::new(1, 13));
        assert_eq!(end, Point::new(13, 1));
        assert_eq!(graph.nodes.len(), 104); // Counted manually :-<
    }

    #[test]
    fn test_pathfinding() {
        let (g1, s1, e1) = parse_input(INPUT_ONE).unwrap();
        let (g2, s2, e2) = parse_input(INPUT_TWO).unwrap();
        let p1 = g1.astar_all_paths(s1, e1).expect("Expected a path");
        let p2 = g2.astar_all_paths(s2, e2).expect("Expected a path");

        // Check that the number of paths are correct.
        assert_eq!(p1.0.len(), 3, "Expected {} paths, got {}", 3, p1.0.len());
        assert_eq!(p2.0.len(), 2, "Expected {} paths, got {}", 3, p2.0.len());
        // Check that the cost of the shortest path is correct.
        assert_eq!(p1.1, 7036, "Expected cost {}, got {}", 7036, p1.1);
        assert_eq!(p2.1, 11048, "Expected cost {}, got {}", 11048, p2.1);
        // Check that the number of unique points in the paths are correct.
        let u1 = unique_points_in_paths(&p1.0);
        let u2 = unique_points_in_paths(&p2.0);
        assert_eq!(u1.len(), 45, "Expected {} points, got {}", 45, u1.len());
        assert_eq!(u2.len(), 64, "Expected {} points, got {}", 64, u2.len());
    }
}
