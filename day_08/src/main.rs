use std::collections::{HashMap, HashSet};

use anyhow::Result;
use itertools::Itertools;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct Coordinate {
    x: i32,
    y: i32,
}

impl Coordinate {
    fn new(x: i32, y: i32) -> Coordinate {
        Coordinate { x, y }
    }

    fn in_bounds(&self, x_limit: usize, y_limit: usize) -> bool {
        self.x >= 0 && self.x < x_limit as i32 && self.y >= 0 && self.y < y_limit as i32
    }
}

#[derive(Debug, PartialEq)]
struct Map {
    width: usize,
    height: usize,
    antennas: Vec<Vec<Coordinate>>,
}

impl Map {
    fn count_unique_antinodes(&self) -> usize {
        let mut antinodes = HashSet::new();

        for antenna_frequency in &self.antennas {
            for (a, b) in antenna_frequency.iter().tuple_combinations() {
                let (c, d) = calculate_antinodes(*a, *b);
                if self.in_bounds(c) {
                    antinodes.insert(c);
                }
                if self.in_bounds(d) {
                    antinodes.insert(d);
                }
            }
        }

        antinodes.len()
    }

    fn count_unique_resonant_antinodes(&self) -> usize {
        let mut antinodes = HashSet::new();

        for antenna_frequency in &self.antennas {
            for (a, b) in antenna_frequency.iter().tuple_combinations() {
                let resonant_antinodes =
                    calculate_resonant_antinodes(*a, *b, self.width, self.height);
                for antinode in resonant_antinodes {
                    antinodes.insert(antinode);
                }
            }
        }

        antinodes.len()
    }

    fn in_bounds(&self, coordinate: Coordinate) -> bool {
        coordinate.in_bounds(self.width, self.height)
    }
}

fn calculate_antinodes(a: Coordinate, b: Coordinate) -> (Coordinate, Coordinate) {
    // Calculate the difference
    let dx = b.x - a.x;
    let dy = b.y - a.y;

    let c = Coordinate::new(b.x + dx, b.y + dy);
    let d = Coordinate::new(a.x - dx, a.y - dy);

    (c, d)
}

//
fn calculate_resonant_antinodes(
    a: Coordinate,
    b: Coordinate,
    x_limit: usize,
    y_limit: usize,
) -> Vec<Coordinate> {
    let mut antinodes = Vec::new();

    // Calculate the difference
    let dx = b.x - a.x;
    let dy = b.y - a.y;

    // See calculate_antinodes, but now an antinode can occur at any coordinate exactly in line with two antennas
    let mut n = 0;
    loop {
        let c = Coordinate::new(a.x + n * dx, a.y + n * dy);
        if !c.in_bounds(x_limit, y_limit) {
            break;
        }
        antinodes.push(c);
        n += 1;
    }

    // Generate points in the negative direction
    n = -1;
    loop {
        let c = Coordinate::new(a.x + n * dx, a.y + n * dy);
        if !c.in_bounds(x_limit, y_limit) {
            break;
        }
        antinodes.push(c);
        n -= 1;
    }

    antinodes
}

#[derive(Debug, Error)]
enum MapParseError {
    #[error("Invalid character in map: {0}")]
    InvalidCharacter(char),
}

fn parse_input(input: &str) -> Result<Map, MapParseError> {
    let mut antennas = HashMap::new();
    let mut width = 0;
    let mut height = 0;

    input.trim().lines().enumerate().try_for_each(|(y, line)| {
        height = height.max(y + 1);
        line.trim().char_indices().try_for_each(|(x, c)| {
            width = width.max(x + 1);
            if is_valid_antenna(c) {
                antennas
                    .entry(c)
                    .or_insert_with(Vec::new)
                    .push(Coordinate::new(x as i32, y as i32));
            } else if is_valid_empty(c) {
                // Do nothing
            } else {
                return Err(MapParseError::InvalidCharacter(c));
            }

            Ok(())
        })
    })?;

    let antennas = antennas.into_values().collect();

    Ok(Map {
        antennas,
        width,
        height,
    })
}

fn is_valid_antenna(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

fn is_valid_empty(c: char) -> bool {
    c == '.'
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;
    let map = parse_input(&input)?;

    // Part 1
    let part_1 = solve_part_1(&map);
    println!("Part 1: {}", part_1);

    // Part 2
    let part_2 = solve_part_2(&map);
    println!("Part 2: {}", part_2);

    Ok(())
}

fn solve_part_1(map: &Map) -> usize {
    map.count_unique_antinodes()
}

fn solve_part_2(map: &Map) -> usize {
    map.count_unique_resonant_antinodes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_map() -> Map {
        let width = 12;
        let height = 12;
        let antennas = vec![
            vec![
                Coordinate::new(8, 1),
                Coordinate::new(5, 2),
                Coordinate::new(7, 3),
                Coordinate::new(4, 4),
            ],
            vec![
                Coordinate::new(6, 5),
                Coordinate::new(8, 8),
                Coordinate::new(9, 9),
            ],
        ];

        Map {
            width,
            height,
            antennas,
        }
    }

    #[test]
    fn test_calculate_antinodes() {
        let a = Coordinate::new(2, 1);
        let b = Coordinate::new(1, 2);

        let (c, d) = calculate_antinodes(a, b);

        assert_eq!(c, Coordinate::new(0, 3));
        assert_eq!(d, Coordinate::new(3, 0));

        // Try reversing the order, it should still work
        let (c, d) = calculate_antinodes(b, a);

        assert_eq!(c, Coordinate::new(3, 0));
        assert_eq!(d, Coordinate::new(0, 3));
    }

    #[test]
    fn test_calculate_resonant_antinodes() {
        let a = Coordinate::new(2, 1);
        let b = Coordinate::new(1, 2);

        let antinodes = calculate_resonant_antinodes(a, b, 4, 4);

        // Cast to a set because the order of the antinodes is not important
        let expected: HashSet<Coordinate> = HashSet::from_iter(vec![
            Coordinate::new(0, 3),
            Coordinate::new(1, 2),
            Coordinate::new(2, 1),
            Coordinate::new(3, 0),
        ]);

        let actual = HashSet::from_iter(antinodes);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_count_unique_antinodes() {
        let map = create_test_map();
        let actual = map.count_unique_antinodes();

        assert_eq!(actual, 14);
    }

    #[test]
    fn test_count_unique_resonant_antinodes() {
        let map = create_test_map();
        let actual = map.count_unique_resonant_antinodes();

        assert_eq!(actual, 34);
    }
}
