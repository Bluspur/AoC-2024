use anyhow::Result;
use priority_queue::PriorityQueue;
use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};
use thiserror::Error;

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
            Direction::North => Self::new(self.x, self.y + 1),
            Direction::South => Self::new(self.x, self.y - 1),
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

    /// Returns whether the direction is perpendicular to the current direction.
    pub fn is_perpendicular(&self, other: Self) -> bool {
        match self {
            Direction::North | Direction::South => {
                matches!(other, Direction::East | Direction::West)
            }
            Direction::East | Direction::West => {
                matches!(other, Direction::North | Direction::South)
            }
        }
    }
}

/// A graph representing a 2D grid of nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    /// A map of points to node ids.
    nodes: HashMap<Point, Node>,
}

impl Graph {
    pub fn new(pos_map: HashSet<Point>) -> Self {
        let mut nodes = HashMap::new();
        for point in &pos_map {
            // Calculate all the possible neighbour points.
            let north = point.neighbour(Direction::North);
            let south = point.neighbour(Direction::South);
            let east = point.neighbour(Direction::East);
            let west = point.neighbour(Direction::West);
            // Create a new node.
            // For each neighbour, check if it is a valid point in the graph.
            let node = Node {
                north: if pos_map.contains(&north) {
                    Some(north)
                } else {
                    None
                },
                south: if pos_map.contains(&south) {
                    Some(south)
                } else {
                    None
                },
                east: if pos_map.contains(&east) {
                    Some(east)
                } else {
                    None
                },
                west: if pos_map.contains(&west) {
                    Some(west)
                } else {
                    None
                },
            };

            nodes.insert(*point, node);
        }
        Self { nodes }
    }

    /// Gets a node at the given point.
    /// Returns Error if the point is not part of the graph.
    pub fn get(&self, point: &Point) -> Result<Node, GraphError> {
        self.nodes
            .get(point)
            .copied()
            .ok_or(GraphError::PointNotFound(*point))
    }

    /// Uses A* algorithm to find the shortest path between two points.
    /// Returns the shortest path as a Vec of points (Start to End).
    /// Entering the node in the current direction will be considered as a cost of 1.
    /// Entering a node 90 degrees to the left or right increases the cost by 1000.
    /// If no path is found, returns an error.
    pub fn find_shortest_path(&self, start: Point, end: Point) -> Result<Path, GraphError> {
        // Check that both start and end points are part of the graph.
        self.get(&start)?;
        self.get(&end)?;
        // Short-circuit if the start and end points are the same.
        if start == end {
            return Ok(Path::new(0, vec![start]));
        }
        // Initialize the agent's direction as East.
        let initial_heading = Direction::East;
        // Initialize the frontier with the start point.
        let mut frontier = PriorityQueue::new();
        frontier.push((start, initial_heading), Reverse(0)); // Reverse the priority to get a min-heap.
        // Initialize a map of nodes with their origins.
        let mut came_from = HashMap::new();
        came_from.insert(start, None);
        // Initialize a map of costs to reach each node.
        let mut cost_so_far = HashMap::new();
        cost_so_far.insert(start, 0);

        // While the frontier is not empty.
        while let Some(((current, heading), _)) = frontier.pop() {
            // If the current point is the end point, end the search.
            if current == end {
                break;
            }

            let node = self.get(&current)?;
            let cost = cost_so_far[&current];
            // Calculate the
            let (left, right) = heading.perpendicular();
            let neighbours = [
                (node.get_neighbour(heading), heading),
                (node.get_neighbour(left), left),
                (node.get_neighbour(right), right),
            ];

            // Iterate over valid neighbours.
            // Filter out any neighbours that are not part of the graph.
            for (next, direction) in neighbours
                .iter()
                .filter_map(|n| n.0.map(|point| (point, n.1)))
            {
                // Calculate the cost to enter the next point.
                let new_cost = cost + if heading == direction { 1 } else { 1001 };

                // If the next point is not in the cost_so_far map or the new cost is less than the current cost.
                if !cost_so_far.contains_key(&next) || new_cost < cost_so_far[&next] {
                    // Update the cost_so_far map with the new cost.
                    cost_so_far.insert(next, new_cost);
                    // Calculate the priority of the next point.
                    let priority = new_cost + next.distance(end);
                    // Push the next point to the frontier.
                    frontier.push((next, direction), Reverse(priority));
                    // Update the came_from map with the current point as the origin of the next point.
                    came_from.insert(next, Some(current));
                }
            }
        }

        // If the end point is not in the came_from map, then no path was found.
        let cost = cost_so_far.get(&end).ok_or(GraphError::NoPathFound)?;

        // Get the path from the end to the start.
        // Reverse the path and store it in a Vec.
        let mut path = Vec::new();
        let mut current = end;
        while let Some(prev) = came_from[&current] {
            path.push(current);
            current = prev;
        }
        path.push(start);
        path.reverse();

        Ok(Path::new(*cost, path))
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    cost: i32,
    points: Vec<Point>,
}

impl Path {
    pub fn new(cost: i32, points: Vec<Point>) -> Self {
        Self { cost, points }
    }

    pub fn length(&self) -> usize {
        self.points.len()
    }

    /// Converts the path to a HashSet of points.
    pub fn to_set(&self) -> HashSet<Point> {
        self.points.iter().copied().collect()
    }
}

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
    /// Returns the neighbour in the given direction.
    pub fn get_neighbour(&self, direction: Direction) -> Option<Point> {
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

    let part_1 = solve_part_1(&input)?;
    println!("Part 1: {}", part_1);

    let part_2 = solve_part_2(&input)?;
    println!("Part 2: {}", part_2);

    Ok(())
}

fn solve_part_1(input: &str) -> Result<i32> {
    let (graph, start, end) = parse_input(input)?;
    let path = graph.find_shortest_path(start, end)?;

    println!("Path: {:?} nodes.", path.points.len());
    println!("Start: {}", start);
    println!("End: {}", end);

    Ok(path.cost)
}

fn solve_part_2(input: &str) -> Result<usize> {
    let (graph, start, end) = parse_input(input)?;
    let paths = graph.find_all_shortest_paths(start, end)?;
    let total = paths.len();

    Ok(total)
}

fn unique_points_in_paths(paths: &[Path]) -> HashSet<Point> {
    paths
        .iter()
        .flat_map(|p| p.points.iter().copied())
        .collect()
}

/// Helper function that prints a graph along with every node in a path.
/// Needs to be told the size since the graph is not stored as a 2D array.
fn print_graph(graph: &Graph, paths: &HashSet<Point>, size: (i32, i32)) {
    for y in 0..size.1 {
        for x in 0..size.0 {
            let point = Point::new(x, y);
            if paths.contains(&point) {
                print!("O");
            } else if graph.nodes.contains_key(&point) {
                print!("·"); // Middle Dot not a period.
            } else {
                print!("$");
            }
        }
        println!();
    }
    println!();
}

pub fn parse_input(input: &str) -> Result<(Graph, Point, Point), GraphError> {
    // Start by normalizing line endings to \n.
    let s = input.replace("\r\n", "\n");
    let mut pos_map = HashSet::new();
    let mut start = None;
    let mut end = None;

    for (y, line) in s.trim().lines().enumerate() {
        for (x, c) in line.trim().char_indices() {
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

    Ok((Graph::new(pos_map), start, end))
}

#[cfg(test)]
mod test {
    use super::*;

    const INPUT: &str = r"
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

    const ALT_INPUT: &str = r"
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
        let (graph, start, end) = parse_input(INPUT).unwrap();

        assert_eq!(start, Point::new(1, 13));
        assert_eq!(end, Point::new(13, 1));
        assert_eq!(graph.nodes.len(), 104); // Counted manually :-<
    }

    #[test]
    fn test_find_shortest_path() {
        let (graph, start, end) = parse_input(INPUT).unwrap();
        let path = graph.find_shortest_path(start, end).unwrap();

        assert_eq!(path.points.len(), 37); // Includes start and end points.
        assert_eq!(path.cost, 7036);
    }

    #[test]
    fn test_find_all_shortest_paths() {
        let (graph, start, end) = parse_input(INPUT).unwrap();
        let paths = graph.find_all_shortest_paths(start, end).unwrap();
        let unique_points = unique_points_in_paths(&paths);
        print_graph(&graph, &unique_points, (15, 15));

        assert_eq!(paths.len(), 3);
    }

    #[test]
    #[ignore]
    fn test_find_all_shortest_paths_alt() {
        let (graph, start, end) = parse_input(ALT_INPUT).unwrap();
        let paths = graph.find_all_shortest_paths(start, end).unwrap();
        let unique_points = unique_points_in_paths(&paths);
        print_graph(&graph, &unique_points, (17, 17));

        assert_eq!(paths.len(), 3);
    }
}
