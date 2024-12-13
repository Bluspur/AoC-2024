use std::{
    collections::{HashMap, HashSet, VecDeque},
    str::FromStr,
};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct Region {
    area: usize,
    perimeter: usize,
}

impl Region {
    pub fn new(area: usize, perimeter: usize) -> Self {
        Region { area, perimeter }
    }
    fn price(&self) -> usize {
        self.area * self.perimeter
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Coordinate {
    x: i32,
    y: i32,
}

impl Coordinate {
    fn new(x: i32, y: i32) -> Self {
        Coordinate { x, y }
    }

    fn in_bounds(&self, width: i32, height: i32) -> bool {
        self.x >= 0 && self.x < width && self.y >= 0 && self.y < height
    }

    fn neighbours_bounded(&self, width: i32, height: i32) -> Vec<Coordinate> {
        let mut neighbours = Vec::new();
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for (dx, dy) in directions.iter() {
            let neighbour = Coordinate::new(self.x + dx, self.y + dy);
            if neighbour.in_bounds(width, height) {
                neighbours.push(neighbour);
            }
        }

        neighbours
    }

    /// All neighbours of a coordinate, including ordinals.
    fn all_neighbours(&self) -> [Coordinate; 8] {
        [
            Coordinate::new(self.x - 1, self.y - 1), // NW
            Coordinate::new(self.x, self.y - 1),     // N
            Coordinate::new(self.x + 1, self.y - 1), // NE
            Coordinate::new(self.x - 1, self.y),     // W
            Coordinate::new(self.x + 1, self.y),     // E
            Coordinate::new(self.x - 1, self.y + 1), // SW
            Coordinate::new(self.x, self.y + 1),     // S
            Coordinate::new(self.x + 1, self.y + 1), // SE
        ]
    }
}

pub struct Graph {
    nodes: HashMap<Coordinate, Node>,
}

impl Graph {
    fn find_regions(&self) -> Regions {
        let mut regions = Vec::new();
        // HashSet of all the coordinates which have been completly handled.
        let mut completed = HashSet::<Coordinate>::new();

        // Loop while there are still nodes in the graph which haven't been handled.
        // Take the first unhandled node as an arbitrary starting point.
        while let Some((start, current)) = self.nodes.iter().find(|(c, _)| !completed.contains(c)) {
            let mut explored = HashSet::<Coordinate>::new();
            let mut queue = VecDeque::new();
            let mut perimeter = 0;
            // This is the token that we will be looking for in this loop
            let token = current.token;

            // Kick start the queue by adding the current node to it
            queue.push_back(*start);

            // Loop while there are any unexplored neighbours
            while let Some(current) = queue.pop_front() {
                // Skip any nodes that we have already examined and found to be part of the region.
                if explored.contains(&current) {
                    continue;
                }
                // Get the node, we can assume it exists.
                let node = self
                    .nodes
                    .get(&current)
                    .expect("Expected node to be present.");
                // Check if the current token matches the one we are looking for.
                if node.token == token {
                    // If it is, then add it to the explored set and queue up its neighbours.
                    explored.insert(current);

                    // If the node is on the edge of the graph, then we can simulate a border with the outside
                    // by calculating the number of out of bounds connections and adding that to the perimeter.
                    // println!("{:?}", node.connections.len());
                    let oob_connections = 4 - node.connections.len();
                    perimeter += oob_connections;

                    for connection in &node.connections {
                        queue.push_back(*connection);
                    }
                } else {
                    // If it is a different token, then we can extend the perimeter by 1.
                    perimeter += 1;
                }
            }

            // Build the new region
            let new_region = Region {
                area: explored.len(),
                perimeter,
            };

            // Add the new region
            regions.push((new_region, explored.clone()));
            // Update the completed Set with all the explored positions.
            completed.extend(explored);
        }

        Regions(regions)
    }

    pub fn new(nodes: HashMap<Coordinate, Node>) -> Self {
        Graph { nodes }
    }
}

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Empty input")]
    EmptyInput,
    #[error("Invalid token: {0}")]
    InvalidToken(char),
}

impl FromStr for Graph {
    type Err = GraphError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(GraphError::EmptyInput);
        }

        let mut nodes = HashMap::new();
        let height = s.lines().count();

        for (y, line) in s.trim().lines().enumerate() {
            // Preemptively trim the line to avoid any issues with whitespace.
            let line = line.trim();
            let width = line.chars().count();
            for (x, c) in line.chars().enumerate() {
                if !c.is_ascii_uppercase() {
                    return Err(GraphError::InvalidToken(c));
                }
                let (x, y) = (x as i32, y as i32);
                let coordinate = Coordinate::new(x, y);
                let neighbours = coordinate.neighbours_bounded(width as i32, height as i32);
                let node = Node::new(c, neighbours);
                nodes.insert(coordinate, node);
            }
        }

        Ok(Graph { nodes })
    }
}

pub struct Node {
    token: char,
    connections: Vec<Coordinate>,
}

impl Node {
    fn new(token: char, connections: Vec<Coordinate>) -> Self {
        Node { token, connections }
    }
}

struct Regions(Vec<(Region, HashSet<Coordinate>)>);

impl Regions {
    fn total_price(&self) -> usize {
        self.0.iter().map(|(r, _)| r.price()).sum()
    }

    fn total_discounted_price(&self) -> usize {
        self.0
            .iter()
            .map(|(r, set)| r.area * count_sides(set))
            .sum()
    }
}

fn count_sides(coords: &HashSet<Coordinate>) -> usize {
    let mut sides = 0;

    let local_grids = coords
        .iter()
        .map(|c| neighbourhood_grid(*c, coords))
        .collect::<Vec<[bool; 9]>>();

    for grid in local_grids {
        if !grid[3] && !grid[1] {
            sides += 1; // NW corner
        }
        if !grid[1] && !grid[5] {
            sides += 1; // NE corner
        }
        if !grid[5] && !grid[7] {
            sides += 1; // SE corner
        }
        if !grid[7] && !grid[3] {
            sides += 1; // SW corner
        }
        if grid[3] && grid[1] && !grid[0] {
            sides += 1; // NW inner
        }
        if grid[1] && grid[5] && !grid[2] {
            sides += 1; // NE inner
        }
        if grid[5] && grid[7] && !grid[8] {
            sides += 1; // SE inner
        }
        if grid[7] && grid[3] && !grid[6] {
            sides += 1; // SW inner
        }
    }

    sides
}

fn neighbourhood_grid(coord: Coordinate, set: &HashSet<Coordinate>) -> [bool; 9] {
    let neighbours = coord.all_neighbours();

    [
        set.contains(&neighbours[0]),
        set.contains(&neighbours[1]),
        set.contains(&neighbours[2]),
        set.contains(&neighbours[3]),
        true,
        set.contains(&neighbours[4]),
        set.contains(&neighbours[5]),
        set.contains(&neighbours[6]),
        set.contains(&neighbours[7]),
    ]
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;
    let graph = input.parse::<Graph>()?;

    // Part 1
    let part_1 = graph.find_regions().total_price();
    println!("Part 1: {}", part_1);

    // Part 2
    let part_2 = graph.find_regions().total_discounted_price();
    println!("Part 2: {}", part_2);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_INPUT: &str = r#"
        AAAA
        BBCD
        BBCC
        EEEC
    "#;

    /// Helper function to create a test graph.
    /// Same as the one described in TEST_INPUT.
    fn create_test_graph() -> Graph {
        /// Helper function to create a coordinate.
        fn c(x: i32, y: i32) -> Coordinate {
            Coordinate::new(x, y)
        }

        /// Helper function to create a node.
        fn n(token: char, connections: Vec<Coordinate>) -> Node {
            Node::new(token, connections)
        }

        let nodes = vec![
            (c(0, 0), n('A', vec![c(0, 1), c(1, 0)])),
            (c(1, 0), n('A', vec![c(0, 0), c(1, 1), c(2, 0)])),
            (c(2, 0), n('A', vec![c(1, 0), c(2, 1), c(3, 0)])),
            (c(3, 0), n('A', vec![c(2, 0), c(3, 1)])),
            (c(0, 1), n('B', vec![c(0, 0), c(1, 1), c(0, 2)])),
            (c(1, 1), n('B', vec![c(0, 1), c(1, 2), c(2, 1), c(1, 0)])),
            (c(2, 1), n('C', vec![c(1, 1), c(2, 0), c(3, 1), c(2, 2)])),
            (c(3, 1), n('D', vec![c(2, 1), c(3, 0), c(3, 2)])),
            (c(0, 2), n('B', vec![c(0, 1), c(1, 2), c(0, 3)])),
            (c(1, 2), n('B', vec![c(0, 2), c(2, 2), c(1, 1), c(1, 3)])),
            (c(2, 2), n('C', vec![c(1, 2), c(3, 2), c(2, 1), c(2, 3)])),
            (c(3, 2), n('C', vec![c(2, 2), c(3, 1), c(3, 3)])),
            (c(0, 3), n('E', vec![c(0, 2), c(1, 3)])),
            (c(1, 3), n('E', vec![c(0, 3), c(1, 2), c(2, 3)])),
            (c(2, 3), n('E', vec![c(1, 3), c(2, 2), c(3, 3)])),
            (c(3, 3), n('C', vec![c(2, 3), c(3, 2)])),
        ];

        Graph::new(nodes.into_iter().collect())
    }

    #[test]
    fn test_parse_graph() {
        let graph = TEST_INPUT.parse::<Graph>().unwrap();

        assert_eq!(graph.nodes.len(), 16);
    }

    #[test]
    fn test_count_sides() {
        let coords = [
            Coordinate::new(0, 0),
            Coordinate::new(1, 0),
            Coordinate::new(2, 0),
            Coordinate::new(3, 0),
        ]
        .into();
        let expected = 4;
        let actual = count_sides(&coords);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_solve_part_2() {
        let graph = create_test_graph();
        let regions = graph.find_regions();
        let expected = 80;

        assert_eq!(regions.total_discounted_price(), expected);
    }

    #[test]
    fn test_find_regions() {
        let graph = create_test_graph();
        let regions = graph.find_regions();

        assert_eq!(regions.0.len(), 5);
    }

    #[test]
    fn test_total_price() {
        let graph = create_test_graph();
        let regions = graph.find_regions();
        let expected = 140;
        let actual = regions.total_price();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calculate_price() {
        let regions = vec![
            (Region::new(12, 18), 216),
            (Region::new(4, 8), 32),
            (Region::new(14, 28), 392),
            (Region::new(10, 18), 180),
            (Region::new(13, 20), 260),
            (Region::new(11, 20), 220),
            (Region::new(1, 4), 4),
            (Region::new(13, 18), 234),
            (Region::new(14, 22), 308),
            (Region::new(5, 12), 60),
            (Region::new(3, 8), 24),
        ];

        for (region, expected_price) in regions {
            assert_eq!(region.price(), expected_price);
        }
    }
}
