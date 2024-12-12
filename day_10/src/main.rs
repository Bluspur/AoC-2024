use std::collections::{hash_map::Entry, HashMap, HashSet, VecDeque};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
enum MapError {
    #[error("Tried to access an OoB position {0:?}")]
    OutOfBoundsError(Coordinate),
}

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

#[derive(Debug, PartialEq, Eq)]
struct Node {
    paths: usize,
    origins: HashSet<Coordinate>,
}

impl Node {
    fn new() -> Self {
        Node {
            paths: 0,
            origins: HashSet::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Map {
    width: usize,
    height: usize,
    inner: Vec<Vec<usize>>,
}

impl Map {
    fn get(&self, pos: Coordinate) -> Result<usize, MapError> {
        if pos.x < self.width && pos.y < self.height {
            Ok(self.inner[pos.y][pos.x])
        } else {
            Err(MapError::OutOfBoundsError(pos))
        }
    }
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

    fn get_neighbours(&self, pos: Coordinate) -> [Option<Coordinate>; 4] {
        let within_bounds = |x, y| {
            if x < self.width && y < self.height {
                Some(Coordinate::new(x, y))
            } else {
                None
            }
        };

        [
            pos.y.checked_sub(1).map(|y| Coordinate::new(pos.x, y)),
            within_bounds(pos.x, pos.y + 1),
            pos.x.checked_sub(1).map(|x| Coordinate::new(x, pos.y)),
            within_bounds(pos.x + 1, pos.y),
        ]
    }

    fn count_trails(&self) -> usize {
        let mut counter = 0;
        let trailheads = self.get_trailheads();
        for trailhead in trailheads {
            counter += bfs_count_trails(&self, trailhead);
        }
        counter
    }
}

fn bfs_count_trails(map: &Map, origin: Coordinate) -> usize {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut counter = 0;

    queue.push_back(origin);
    visited.insert(origin);

    while let Some(pos) = queue.pop_front() {
        let cur_val = map.inner[pos.y][pos.x];

        for n_pos in map.get_neighbours(pos) {
            let Some(n_pos) = n_pos else {
                continue;
            };
            if n_pos.x < map.width && n_pos.y < map.height && !visited.contains(&n_pos) {
                let n_val = map.inner[n_pos.y][n_pos.x];
                if n_val == cur_val + 1 {
                    if n_val == 9 {
                        counter += 1;
                    } else {
                        queue.push_back(n_pos);
                    }
                    visited.insert(n_pos);
                }
            }
        }
    }

    counter
}

fn valid_neighbours(a: usize, b: usize) -> bool {
    a + 1 == b
}

fn bubble_counter(graph: &mut HashMap<Coordinate, Node>, bubble_origin: Coordinate, value: usize) {
    // It's a waste of resources to continue if the bubble value is 0.
    if value < 1 {
        return;
    }

    let mut queue = VecDeque::new();
    queue.push_back(bubble_origin);

    while let Some(current) = queue.pop_front() {
        let node = graph
            .get_mut(&current)
            .expect("Expected origin to be present.");

        node.paths += value;

        for origin in node.origins.iter() {
            queue.push_back(*origin);
        }
    }
}

fn count_all_paths(map: &Map) -> Result<usize, MapError> {
    let mut explored: HashMap<Coordinate, Node> = HashMap::new();
    let mut queue = VecDeque::new();
    let mut counter = 0;

    let all_trailheads = map.get_trailheads();

    // Iterate through all possible origin points (trailheads).
    for trailhead in all_trailheads {
        // Add the trailhead to the queue.
        queue.push_back((None, trailhead));

        while let Some((origin, current)) = queue.pop_front() {
            let current_value = map.get(current)?;

            // Firstly check if the current position has already been explored.
            match explored.entry(current) {
                // The current position has already been explored.
                Entry::Occupied(mut e) => {
                    let path_count = e.get().paths;
                    // We need to update the origins of the current node.
                    e.get_mut().origins.insert(origin.unwrap());
                    // If it is already present, then we can perpetuate the value up the graph.
                    bubble_counter(&mut explored, origin.unwrap(), path_count);
                }
                Entry::Vacant(e) => {
                    // Prepare a new node.
                    let mut node = Node::new();

                    if current_value == 9 {
                        // We can assume that origin is Some
                        let origin = origin.unwrap();
                        // End nodes have only a single path. (To themselves).
                        node.paths = 1;
                        node.origins.insert(origin);
                        e.insert(node);
                        // We also need to bubble when we find an endpoint.
                        bubble_counter(&mut explored, origin, 1);
                    } else {
                        // Get all possible neighbours (including possible OoB).
                        let all_neighbours = map.get_neighbours(current);

                        // Filter out any OoB neighbours.
                        for neighbour in all_neighbours.iter().filter_map(|n| *n) {
                            let neighbour_value = map.get(neighbour)?;

                            // Ignore any neighbours that don't follow the valid path rules.
                            if !valid_neighbours(current_value, neighbour_value) {
                                continue;
                            }

                            // Add any valid neighbours to the exploration queue.
                            queue.push_back((Some(current), neighbour));
                        }

                        if let Some(origin) = origin {
                            node.origins.insert(origin);
                        }

                        e.insert(node);
                    }
                }
            }
        }

        // We are safe to assume that the origin is in the explored map.
        let origin_path_count = explored.get(&trailhead).unwrap().paths;
        counter += origin_path_count;
    }

    Ok(counter)
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
    count_all_paths(map)
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
    fn test_bfs_count_trails() {
        let map = create_large_test_map();
        let origin = Coordinate::new(2, 0);
        let expected = 5;
        let actual = bfs_count_trails(&map, origin);

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
        let actual = count_all_paths(&map).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_solve_part_2() {
        let map = create_large_test_map();
        let expected = 81;
        let actual = count_all_paths(&map).unwrap();

        assert_eq!(expected, actual);
    }
}
