use std::collections::HashSet;

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PosState {
    O, // Open
    X, // Closed
}

/// 2d top down map of the patrol area.
/// Origin point in the upper left position.
/// Assumed to be a regular in shape, i.e. all rows are the same length
#[derive(Debug, PartialEq, Eq)]
struct Map(Vec<Vec<PosState>>);

impl Map {
    /// Get the number of rows
    fn height(&self) -> usize {
        self.0.len()
    }
    /// Get the number of columns
    fn width(&self) -> usize {
        self.0[0].len()
    }
    fn get_position(&self, x: usize, y: usize) -> Option<PosState> {
        // Check for OOB
        if x >= self.width() || y >= self.height() {
            return None;
        }

        Some(self.0[y][x])
    }
}

enum Translation {
    Add((usize, usize)),
    Subtract((usize, usize)),
}

impl Translation {
    /// Consumes the translation and returns the new 2d coordinate, assuming it didn't underflow.
    fn apply_translation(self, x: usize, y: usize) -> Option<(usize, usize)> {
        match self {
            Translation::Add((dx, dy)) => Some((x + dx, y + dy)),
            Translation::Subtract((dx, dy)) => Some((x.checked_sub(dx)?, y.checked_sub(dy)?)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Heading {
    N,
    E,
    W,
    S,
}

impl Heading {
    fn turn(self) -> Self {
        match self {
            Heading::N => Heading::E,
            Heading::E => Heading::S,
            Heading::W => Heading::N,
            Heading::S => Heading::W,
        }
    }
    fn get_translation(&self) -> Translation {
        match self {
            Heading::N => Translation::Subtract((0, 1)),
            Heading::E => Translation::Add((1, 0)),
            Heading::W => Translation::Subtract((1, 0)),
            Heading::S => Translation::Add((0, 1)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Guard {
    position: (usize, usize),
    heading: Heading,
}

impl Guard {
    fn advance(self, map: &Map) -> Option<Guard> {
        let (x, y) = self.position;
        let translation = self.heading.get_translation();
        // Returns None if the new position underflows, represents leaving the map.
        let (new_x, new_y) = translation.apply_translation(x, y)?;
        // Returns None if the position was out of bounds, represents leaving the map.
        let position_type = map.get_position(new_x, new_y)?;

        match position_type {
            // If the position is open, then enter the new position.
            PosState::O => Some(Guard {
                position: (new_x, new_y),
                ..self
            }),
            // If the position is closed, then turn 90 degrees.
            PosState::X => Some(Guard {
                heading: self.heading.turn(),
                ..self
            }),
        }
    }
}

#[derive(Debug, Error)]
enum ParsingError {
    #[error("Invalid Character: {0}")]
    InvalidCharacter(char),
    #[error("No guard found in input")]
    MissingGuard,
}

fn parse_input(input: &str) -> Result<(Map, Guard), ParsingError> {
    // Split the input into rows
    let lines = input.trim().lines();

    let mut map = Vec::new();
    let mut guard = None;

    for (y, line) in lines.enumerate() {
        let mut row = Vec::new();
        for (x, symbol) in line.trim().char_indices() {
            match symbol {
                '.' => row.push(PosState::O),
                '#' => row.push(PosState::X),
                '^' => {
                    row.push(PosState::O);
                    guard = Some(Guard {
                        position: (x, y),
                        heading: Heading::N,
                    });
                }
                _ => return Err(ParsingError::InvalidCharacter(symbol)),
            }
        }
        map.push(row);
    }

    let Some(guard) = guard else {
        return Err(ParsingError::MissingGuard);
    };

    Ok((Map(map), guard))
}

fn main() -> Result<()> {
    let raw_input = std::fs::read_to_string("input.txt")?;
    let (map, guard) = parse_input(&raw_input)?;

    // Part 1
    let part_1 = solve_part_1(&map, guard);
    println!("Part 1: {}", part_1);

    Ok(())
}

fn solve_part_1(map: &Map, mut guard: Guard) -> usize {
    // Unique visited positions.
    let mut visited = HashSet::new();
    // Initialize visited with the starting position.
    visited.insert(guard.position);

    while let Some(new_guard) = guard.advance(map) {
        visited.insert(new_guard.position);

        // println!(
        //     "({},{}), {:?}",
        //     guard.position.0, guard.position.1, guard.heading
        // );

        guard = new_guard
    }

    visited.len()
}

fn solve_part_2(map: &Map, mut guard: Guard) -> usize {
    let mut corners = Vec::new();
    let mut path = vec![guard.position];

    let mut i = 0;
    while let Some(new_guard) = guard.advance(map) {
        path.push(new_guard.position);

        if new_guard.heading != guard.heading {
            println!("Corner ({},{})", new_guard.position.0, new_guard.position.1);
            corners.push((new_guard, i));
        }

        guard = new_guard;
        i += 1;
    }

    0
}

/// B is assumed to the right angle
fn calc_opposite_corner(a: (usize, usize), b: (usize, usize), c: (usize, usize)) -> (usize, usize) {
    (a.0 + c.0 - b.0, a.1 + c.1 - b.1)
}

fn right_angle_corner(
    a: (usize, usize),
    b: (usize, usize),
    c: (usize, usize),
) -> Option<(usize, usize)> {
    let ab_sq = sq_len(a, b);
    let bc_sq = sq_len(b, c);
    let ca_sq = sq_len(c, a);

    if ab_sq + bc_sq == ca_sq {
        Some(b)
    } else if ab_sq + ca_sq == bc_sq {
        Some(a)
    } else if bc_sq + ca_sq == ab_sq {
        Some(c)
    } else {
        None
    }
}

fn sq_len(a: (usize, usize), b: (usize, usize)) -> usize {
    let x_sq = if b.0 >= a.0 {
        (b.0 - a.0).pow(2)
    } else {
        (a.0 - b.0).pow(2)
    };
    let y_sq = if b.1 >= a.1 {
        (b.1 - a.1).pow(2)
    } else {
        (a.1 - b.1).pow(2)
    };
    x_sq + y_sq
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_INPUT: &str = r#"        
        ....#.....
        .........#
        ..........
        ..#.......
        .......#..
        ..........
        .#..^.....
        ........#.
        #.........
        ......#...
"#;

    fn create_test_map() -> Map {
        use PosState::*;
        Map(vec![
            vec![O, O, O, O, X, O, O, O, O, O],
            vec![O, O, O, O, O, O, O, O, O, X],
            vec![O, O, O, O, O, O, O, O, O, O],
            vec![O, O, X, O, O, O, O, O, O, O],
            vec![O, O, O, O, O, O, O, X, O, O],
            vec![O, O, O, O, O, O, O, O, O, O],
            vec![O, X, O, O, O, O, O, O, O, O],
            vec![O, O, O, O, O, O, O, O, X, O],
            vec![X, O, O, O, O, O, O, O, O, O],
            vec![O, O, O, O, O, O, X, O, O, O],
        ])
    }

    #[test]
    fn test_parse_input() {
        let expected_map = create_test_map();
        let expected_guard = Guard {
            position: (4, 6),
            heading: Heading::N,
        };

        let (actual_map, actual_guard) = parse_input(TEST_INPUT).unwrap();

        assert_eq!(expected_map, actual_map);
        assert_eq!(expected_guard, actual_guard);
    }

    #[test]
    fn test_get_position() {
        let map = create_test_map();

        // OOB East
        assert_eq!(None, map.get_position(10, 0));
        // OOB South
        assert_eq!(None, map.get_position(0, 10));
        // Valid Open
        assert_eq!(Some(PosState::O), map.get_position(0, 0));
        // Valid Closed
        assert_eq!(Some(PosState::X), map.get_position(4, 0));
    }

    #[test]
    fn test_solve_part_1() {
        let map = create_test_map();
        let guard = Guard {
            position: (4, 6),
            heading: Heading::N,
        };

        let actual = solve_part_1(&map, guard);

        assert_eq!(41, actual);
    }

    #[test]
    fn test_solve_part_2() {
        let map = create_test_map();
        let guard = Guard {
            position: (4, 6),
            heading: Heading::N,
        };

        let actual = solve_part_2(&map, guard);

        assert_eq!(6, actual);
    }

    #[test]
    fn test_is_right_angle_triangle() {
        let (a, b, c) = ((4, 1), (8, 1), (8, 6));

        assert_eq!(right_angle_corner(a, b, c), Some(b));
        assert_eq!(right_angle_corner(b, a, c), Some(b));
        assert_ne!(right_angle_corner(a, b, c), Some(a));
    }
}
