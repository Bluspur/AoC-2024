use std::collections::{hash_map::Entry, HashMap, HashSet, VecDeque};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
enum MapError {
    #[error("Tried to access an OoB position {0}")]
    OutOfBounds(Coordinate),
    #[error("Tried to access a position with no origin {0}")]
    MissingOrigin(Coordinate),
    #[error("Tried to access a position which was missing {0}")]
    MissingPosition(Coordinate),
}

/// 2d coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl Coordinate {
    fn new(x: usize, y: usize) -> Self {
        Coordinate { x, y }
    }
}

impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Node representation for the graph.
#[derive(Debug, Default, PartialEq, Eq)]
struct Node {
    paths: usize,
    origins: HashSet<Coordinate>,
}

/// 2d representation of a map of integers.
#[derive(Debug, PartialEq, Eq)]
struct Map {
    /// Width of the map.
    width: usize,
    /// Height of the map.
    height: usize,
    /// Inner representation of the map.
    inner: Vec<Vec<usize>>,
}

impl Map {
    /// Get the integer value at a given position.
    fn get(&self, pos: Coordinate) -> Result<usize, MapError> {
        if pos.x < self.width && pos.y < self.height {
            Ok(self.inner[pos.y][pos.x])
        } else {
            Err(MapError::OutOfBounds(pos))
        }
    }
    /// Get all trailheads (values of 0) on the map.
    fn get_trailheads(&self) -> HashSet<Coordinate> {
        let mut trailheads = HashSet::new();

        for y in 0..self.height {
            for x in 0..self.width {
                if self.inner[y][x] == 0 {
                    let pos = Coordinate::new(x, y);
                    trailheads.insert(pos);
                }
            }
        }

        trailheads
    }

    /// Get all neighbours of a given position.
    /// Returns an array of 4 options, where None represents an OoB position.
    fn get_neighbours(&self, pos: Coordinate) -> [Option<Coordinate>; 4] {
        let within_bounds = |x, y| {
            if x < self.width && y < self.height {
                Some(Coordinate::new(x, y))
            } else {
                None
            }
        };

        [
            pos.y.checked_sub(1).map(|y| Coordinate::new(pos.x, y)), // North
            within_bounds(pos.x, pos.y + 1),                         // South
            pos.x.checked_sub(1).map(|x| Coordinate::new(x, pos.y)), // West
            within_bounds(pos.x + 1, pos.y),                         // East
        ]
    }

    /// Count the number of reachables trails from all trailheads.
    fn count_trails(&self) -> usize {
        let mut counter = 0;
        let trailheads = self.get_trailheads();
        for trailhead in trailheads {
            counter += self.count_valid_trails_from_trailhead(trailhead);
        }
        counter
    }

    /// Count the number of valid trails from a given trailhead.
    /// Returns the number of valid trails.
    /// Uses a BFS approach.
    fn count_valid_trails_from_trailhead(&self, origin: Coordinate) -> usize {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut counter = 0;

        queue.push_back(origin);
        visited.insert(origin);

        // Loop until the queue is empty.
        while let Some(pos) = queue.pop_front() {
            // Cache the value at the current position.
            let cur_val = self.inner[pos.y][pos.x];

            // Loop through valid neighbours.
            for n_pos in self.get_neighbours(pos) {
                // If the neighbour is OoB, we can ignore it.
                let Some(n_pos) = n_pos else {
                    continue;
                };

                // If the neighbour has not been visited, we can explore it.
                if !visited.contains(&n_pos) {
                    // Cache the value at the neighbour position.
                    let n_val = self.inner[n_pos.y][n_pos.x];
                    // If the neighbour is a valid step on the path, we can add it to the queue.
                    if n_val == cur_val + 1 {
                        // If the neighbour is an endpoint, we can increment the counter.
                        if n_val == 9 {
                            counter += 1;
                        } else {
                            queue.push_back(n_pos);
                        }
                        // Mark the neighbour as visited.
                        visited.insert(n_pos);
                    }
                }
            }
        }

        counter
    }

    /// Takes in a map and counts all possible paths from all trailheads to all endpoints.
    /// Returns the total number of paths.
    /// Considers every possible valid trail from every trailhead to every endpoint.
    fn count_all_valid_trails(&self) -> Result<usize, MapError> {
        let mut explored: HashMap<Coordinate, Node> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut counter = 0;

        // Get all possible trailheads.
        let all_trailheads = self.get_trailheads();

        // Iterate through all possible origin points (trailheads).
        for trailhead in all_trailheads {
            // Add the trailhead to the queue.
            queue.push_back((None, trailhead));

            // Loop until the queue is empty.
            while let Some((origin, current)) = queue.pop_front() {
                // Cache the value at the current position.
                let current_value = self.get(current)?;

                // Check if the current position has already been explored.
                match explored.entry(current) {
                    Entry::Occupied(mut e) => {
                        // Cache the number of valid paths from the current node.
                        let path_count = e.get().paths;
                        // Cover the case where the current node is an trailhead.
                        // This should theoretically never happen.
                        if let Some(origin) = origin {
                            // We need to update the origins of the current node.
                            e.get_mut().origins.insert(origin);
                            // If it is already present, then we can perpetuate the value up the graph.
                            bubble_counter(&mut explored, origin, path_count)?;
                        } else {
                            // Early return an error if the origin is missing.
                            return Err(MapError::MissingOrigin(current));
                        }
                    }
                    Entry::Vacant(e) => {
                        // Prepare a new node.
                        let mut node = Node::default();

                        // Diverge based on if the position is an endpoint or not.
                        if current_value == 9 {
                            if let Some(origin) = origin {
                                // End nodes have only a single path. (To themselves).
                                node.paths = 1;
                                node.origins.insert(origin);
                                e.insert(node);
                                // We also need to bubble when we find an endpoint.
                                bubble_counter(&mut explored, origin, 1)?;
                            } else {
                                // Early return an error if the origin is missing.
                                return Err(MapError::MissingOrigin(current));
                            }
                        } else {
                            // Get all possible neighbours (including possible OoB).
                            let all_neighbours = self.get_neighbours(current);

                            // Filter out any OoB neighbours.
                            for neighbour in all_neighbours.iter().filter_map(|n| *n) {
                                let neighbour_value = self.get(neighbour)?;

                                // Ignore any neighbours that don't follow the valid path rules.
                                if !valid_neighbours(current_value, neighbour_value) {
                                    continue;
                                }

                                // Add any valid neighbours to the exploration queue.
                                queue.push_back((Some(current), neighbour));
                            }

                            // Update the node with the origin.
                            if let Some(origin) = origin {
                                node.origins.insert(origin);
                            }

                            // Insert the node into the explored map.
                            e.insert(node);
                        }
                    }
                }
            }

            // We are safe to assume that the origin is in the explored map.
            let origin_path_count = explored
                .get(&trailhead)
                .ok_or_else(|| MapError::MissingPosition(trailhead))?
                .paths;

            // Add the path count to the total counter.
            counter += origin_path_count;
        }

        Ok(counter)
    }
}

/// Checks if two values are valid neighbours.
/// According to the rules, two values are valid neighbours if they are exactly 1 apart.
fn valid_neighbours(from: usize, to: usize) -> bool {
    from + 1 == to
}

/// Takes in a graph of nodes at a given position and "bubbles up" the path count by the given value.
/// Assumes that the origin is present in the graph.
fn bubble_counter(
    graph: &mut HashMap<Coordinate, Node>,
    bubble_origin: Coordinate,
    value: usize,
) -> Result<(), MapError> {
    if value < 1 {
        // If the value is less than 1, we can ignore it.
        return Ok(());
    }

    let mut queue = VecDeque::new();
    queue.push_back(bubble_origin);

    // Loop through the origins and bubble up the path count.
    while let Some(current) = queue.pop_front() {
        let node = graph
            .get_mut(&current)
            .ok_or_else(|| MapError::MissingOrigin(current))?;

        node.paths += value;

        for origin in node.origins.iter() {
            queue.push_back(*origin);
        }
    }

    Ok(())
}

fn parse_input(input: &str) -> Result<Map> {
    let inner = input
        .trim()
        .lines()
        .map(|line| {
            line.trim()
                .chars()
                .map(|c| c.to_string().parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<Vec<_>>, _>>()?;
    let height = inner.len();
    let width = inner[0].len();

    Ok(Map {
        width,
        height,
        inner,
    })
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;
    let map = parse_input(&input)?;

    // Part 1
    let part_1 = solve_part_1(&map);
    println!("Part 1: {}", part_1);

    // Part 2
    let part_2 = solve_part_2(&map)?;
    println!("Part 2: {}", part_2);

    Ok(())
}

fn solve_part_1(map: &Map) -> usize {
    map.count_trails()
}

fn solve_part_2(map: &Map) -> Result<usize, MapError> {
    map.count_all_valid_trails()
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_INPUT: &str = r#"
        0123
        1234
        8765
        9876
    "#;

    fn create_test_map() -> Map {
        Map {
            height: 4,
            width: 4,
            inner: vec![
                vec![0, 1, 2, 3],
                vec![1, 2, 3, 4],
                vec![8, 7, 6, 5],
                vec![9, 8, 7, 6],
            ],
        }
    }

    fn create_large_test_map() -> Map {
        Map {
            height: 8,
            width: 8,
            inner: vec![
                vec![8, 9, 0, 1, 0, 1, 2, 3],
                vec![7, 8, 1, 2, 1, 8, 7, 4],
                vec![8, 7, 4, 3, 0, 9, 6, 5],
                vec![9, 6, 5, 4, 9, 8, 7, 4],
                vec![4, 5, 6, 7, 8, 9, 0, 3],
                vec![3, 2, 0, 1, 9, 0, 1, 2],
                vec![0, 1, 3, 2, 9, 8, 0, 1],
                vec![1, 0, 4, 5, 6, 7, 3, 2],
            ],
        }
    }

    #[test]
    fn test_parse_map() {
        let expected = create_test_map();
        let actual = parse_input(TEST_INPUT).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_get_neighbours() {
        let map = create_test_map();
        let expected = [
            None,
            Some(Coordinate::new(0, 1)),
            None,
            Some(Coordinate::new(1, 0)),
        ];
        let actual = map.get_neighbours(Coordinate::new(0, 0));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_count_valid_trails_from_trailhead() {
        let map = create_large_test_map();
        let trailhead = Coordinate::new(2, 0);
        let expected = 5;
        let actual = map.count_valid_trails_from_trailhead(trailhead);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_count_trailheads() {
        let map = create_large_test_map();
        let expected = 36;
        let actual = map.count_trails();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_solve_part_2_small() {
        let map = create_test_map();
        let expected = 16;
        let actual = map.count_all_valid_trails().unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_solve_part_2() {
        let map = create_large_test_map();
        let expected = 81;
        let actual = map.count_all_valid_trails().unwrap();

        assert_eq!(expected, actual);
    }
}
