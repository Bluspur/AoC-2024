use std::{
    collections::{HashMap, HashSet, VecDeque},
    str::FromStr,
};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
struct Region {
    area: usize,
    perimeter: usize,
}

impl Region {
    fn price(&self) -> usize {
        self.area * self.perimeter
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Coordinate {
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

    fn neighbours(&self, width: i32, height: i32) -> Vec<Coordinate> {
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
}

struct Graph {
    nodes: HashMap<Coordinate, Node>,
}

impl Graph {
    fn find_regions(self) -> Regions {
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

            println!("Region {}: {:?}", token, new_region);

            // Add the new region
            regions.push(new_region);
            // Update the completed Set with all the explored positions.
            completed.extend(explored);
        }

        Regions(regions)
    }
}

#[derive(Debug, Error)]
enum GraphError {
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
                let neighbours = coordinate.neighbours(width as i32, height as i32);
                println!("{:?} -> {:?}", coordinate, neighbours.len());
                let node = Node::new(c, neighbours);
                nodes.insert(coordinate, node);
            }
        }

        Ok(Graph { nodes })
    }
}

struct Regions(Vec<Region>);

impl Regions {
    fn total_price(&self) -> usize {
        self.0.iter().map(|r| r.price()).sum()
    }
}

struct Node {
    token: char,
    connections: Vec<Coordinate>,
}

impl Node {
    fn new(token: char, connections: Vec<Coordinate>) -> Self {
        Node { token, connections }
    }
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;
    let graph = input.parse::<Graph>()?;

    // Part 1
    let part_1 = graph.find_regions().total_price();
    println!("Part 1: {}", part_1);

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

    fn create_test_graph() -> Graph {
        Graph {
            nodes: [
                (
                    Coordinate::new(0, 0),
                    Node::new('A', vec![Coordinate::new(0, 1), Coordinate::new(1, 0)]),
                ),
                (
                    Coordinate::new(1, 0),
                    Node::new('A', vec![
                        Coordinate::new(0, 0),
                        Coordinate::new(1, 1),
                        Coordinate::new(2, 0),
                    ]),
                ),
                (
                    Coordinate::new(2, 0),
                    Node::new('A', vec![
                        Coordinate::new(1, 0),
                        Coordinate::new(2, 1),
                        Coordinate::new(3, 0),
                    ]),
                ),
                (
                    Coordinate::new(3, 0),
                    Node::new('A', vec![Coordinate::new(2, 0), Coordinate::new(3, 1)]),
                ),
                (
                    Coordinate::new(0, 1),
                    Node::new('B', vec![
                        Coordinate::new(0, 0),
                        Coordinate::new(1, 1),
                        Coordinate::new(0, 2),
                    ]),
                ),
                (
                    Coordinate::new(1, 1),
                    Node::new('B', vec![
                        Coordinate::new(0, 1),
                        Coordinate::new(1, 2),
                        Coordinate::new(2, 1),
                        Coordinate::new(1, 0),
                    ]),
                ),
                (
                    Coordinate::new(2, 1),
                    Node::new('C', vec![
                        Coordinate::new(1, 1),
                        Coordinate::new(2, 0),
                        Coordinate::new(3, 1),
                        Coordinate::new(2, 2),
                    ]),
                ),
                (
                    Coordinate::new(3, 1),
                    Node::new('D', vec![
                        Coordinate::new(2, 1),
                        Coordinate::new(3, 0),
                        Coordinate::new(3, 2),
                    ]),
                ),
                (
                    Coordinate::new(0, 2),
                    Node::new('B', vec![
                        Coordinate::new(0, 1),
                        Coordinate::new(1, 2),
                        Coordinate::new(0, 3),
                    ]),
                ),
                (
                    Coordinate::new(1, 2),
                    Node::new('B', vec![
                        Coordinate::new(0, 2),
                        Coordinate::new(2, 2),
                        Coordinate::new(1, 1),
                        Coordinate::new(1, 3),
                    ]),
                ),
                (
                    Coordinate::new(2, 2),
                    Node::new('C', vec![
                        Coordinate::new(1, 2),
                        Coordinate::new(3, 2),
                        Coordinate::new(2, 1),
                        Coordinate::new(2, 3),
                    ]),
                ),
                (
                    Coordinate::new(3, 2),
                    Node::new('C', vec![
                        Coordinate::new(2, 2),
                        Coordinate::new(3, 1),
                        Coordinate::new(3, 3),
                    ]),
                ),
                (
                    Coordinate::new(0, 3),
                    Node::new('E', vec![Coordinate::new(0, 2), Coordinate::new(1, 3)]),
                ),
                (
                    Coordinate::new(1, 3),
                    Node::new('E', vec![
                        Coordinate::new(0, 3),
                        Coordinate::new(1, 2),
                        Coordinate::new(2, 3),
                    ]),
                ),
                (
                    Coordinate::new(2, 3),
                    Node::new('E', vec![
                        Coordinate::new(1, 3),
                        Coordinate::new(2, 2),
                        Coordinate::new(3, 3),
                    ]),
                ),
                (
                    Coordinate::new(3, 3),
                    Node::new('C', vec![Coordinate::new(2, 3), Coordinate::new(3, 2)]),
                ),
            ]
            .into(),
        }
    }

    #[test]
    fn parse_graph() {
        let graph = TEST_INPUT.parse::<Graph>().unwrap();

        assert_eq!(graph.nodes.len(), 16);
    }

    #[test]
    fn find_regions() {
        let graph = create_test_graph();
        let regions = graph.find_regions();

        assert_eq!(regions.0.len(), 5);
    }

    #[test]
    fn total_price() {
        let graph = create_test_graph();
        let regions = graph.find_regions();
        let expected = 140;
        let actual = regions.total_price();

        assert_eq!(actual, expected);
    }

    #[test]
    fn calculate_price() {
        let regions = vec![
            (
                Region {
                    area: 12,
                    perimeter: 18,
                },
                216,
            ),
            (
                Region {
                    area: 4,
                    perimeter: 8,
                },
                32,
            ),
            (
                Region {
                    area: 14,
                    perimeter: 28,
                },
                392,
            ),
            (
                Region {
                    area: 10,
                    perimeter: 18,
                },
                180,
            ),
            (
                Region {
                    area: 13,
                    perimeter: 20,
                },
                260,
            ),
            (
                Region {
                    area: 11,
                    perimeter: 20,
                },
                220,
            ),
            (
                Region {
                    area: 1,
                    perimeter: 4,
                },
                4,
            ),
            (
                Region {
                    area: 13,
                    perimeter: 18,
                },
                234,
            ),
            (
                Region {
                    area: 14,
                    perimeter: 22,
                },
                308,
            ),
            (
                Region {
                    area: 5,
                    perimeter: 12,
                },
                60,
            ),
            (
                Region {
                    area: 3,
                    perimeter: 8,
                },
                24,
            ),
        ];

        for (region, expected) in regions {
            assert_eq!(region.price(), expected);
        }
    }
}
