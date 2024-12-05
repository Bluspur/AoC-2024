use std::str::FromStr;

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid character: {0}")]
    InvalidCharacter(char),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Letters {
    X,
    M,
    A,
    S,
}

#[derive(Debug, PartialEq)]
pub struct Grid(Vec<Vec<Letters>>);

impl Grid {
    /// Count the number of XMAS words in the grid.
    /// An XMAS word is a word that starts with an X and is followed by M, A, S in any direction.
    pub fn count_xmas(&self) -> u32 {
        let mut count = 0;

        for (y, row) in self.0.iter().enumerate() {
            for (x, letter) in row.iter().enumerate() {
                // We only care about X's since they represent the start of the word.
                if letter == &Letters::X {
                    // Check right
                    if x + 4 <= row.len() {
                        if row[x + 1] == Letters::M
                            && row[x + 2] == Letters::A
                            && row[x + 3] == Letters::S
                        {
                            count += 1;
                        }
                    }
                    // Check Left
                    if x >= 3 {
                        if row[x - 1] == Letters::M
                            && row[x - 2] == Letters::A
                            && row[x - 3] == Letters::S
                        {
                            count += 1;
                        }
                    }
                    // Check Down
                    if y + 4 <= self.0.len() {
                        if self.0[y + 1][x] == Letters::M
                            && self.0[y + 2][x] == Letters::A
                            && self.0[y + 3][x] == Letters::S
                        {
                            count += 1;
                        }
                    }
                    // Check Up
                    if y >= 3 {
                        if self.0[y - 1][x] == Letters::M
                            && self.0[y - 2][x] == Letters::A
                            && self.0[y - 3][x] == Letters::S
                        {
                            count += 1;
                        }
                    }
                    // Check Diagonal Up Right
                    if x + 4 <= row.len() && y >= 3 {
                        if self.0[y - 1][x + 1] == Letters::M
                            && self.0[y - 2][x + 2] == Letters::A
                            && self.0[y - 3][x + 3] == Letters::S
                        {
                            count += 1;
                        }
                    }
                    // Check Diagonal Up Left
                    if x >= 3 && y >= 3 {
                        if self.0[y - 1][x - 1] == Letters::M
                            && self.0[y - 2][x - 2] == Letters::A
                            && self.0[y - 3][x - 3] == Letters::S
                        {
                            count += 1;
                        }
                    }
                    // Check Diagonal Down Right
                    if x + 4 <= row.len() && y + 4 <= self.0.len() {
                        if self.0[y + 1][x + 1] == Letters::M
                            && self.0[y + 2][x + 2] == Letters::A
                            && self.0[y + 3][x + 3] == Letters::S
                        {
                            count += 1;
                        }
                    }
                    // Check Diagonal Down Left
                    if x >= 3 && y + 4 <= self.0.len() {
                        if self.0[y + 1][x - 1] == Letters::M
                            && self.0[y + 2][x - 2] == Letters::A
                            && self.0[y + 3][x - 3] == Letters::S
                        {
                            count += 1;
                        }
                    }
                }
            }
        }

        count
    }

    /// Count the number of crossed Mas words in the grid
    /// A crossed Mas word is a word that has A in the middle and is crossed diagonally by M and S
    pub fn count_x_mas(&self) -> u32 {
        let mut count = 0;

        for (y, row) in self.0.iter().enumerate() {
            for (x, letter) in row.iter().enumerate() {
                // We only care about A's since they represent the middle of the word.
                if letter == &Letters::A {
                    // Crosses cannot be on the edge of the grid
                    if x == 0 || x == row.len() - 1 || y == 0 || y == self.0.len() - 1 {
                        continue;
                    }

                    let (nw, ne, sw, se) = (
                        self.0[y - 1][x - 1],
                        self.0[y - 1][x + 1],
                        self.0[y + 1][x - 1],
                        self.0[y + 1][x + 1],
                    );

                    // We can immediately skip if any of the diagonals are A's or X's
                    if nw == Letters::A
                        || nw == Letters::X
                        || ne == Letters::A
                        || ne == Letters::X
                        || sw == Letters::A
                        || sw == Letters::X
                        || se == Letters::A
                        || se == Letters::X
                    {
                        continue;
                    }

                    if nw == se || ne == sw {
                        continue;
                    }

                    count += 1;
                }
            }
        }

        count
    }
}

impl FromStr for Grid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = Vec::new();

        for line in s.lines() {
            let mut row = Vec::new();
            for c in line.chars() {
                let letter = match c {
                    'X' => Letters::X,
                    'M' => Letters::M,
                    'A' => Letters::A,
                    'S' => Letters::S,
                    _ => return Err(Error::InvalidCharacter(c)),
                };
                row.push(letter);
            }
            grid.push(row);
        }

        Ok(Grid(grid))
    }
}

fn main() -> Result<()> {
    let raw_input = std::fs::read_to_string("input.txt")?;
    let grid = raw_input.parse::<Grid>()?;

    let part_1 = solve_part_1(&grid);
    println!("Part 1: {}", part_1);

    let part_2 = solve_part_2(&grid);
    println!("Part 2: {}", part_2);

    Ok(())
}

fn solve_part_1(grid: &Grid) -> u32 {
    grid.count_xmas()
}

fn solve_part_2(grid: &Grid) -> u32 {
    grid.count_x_mas()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Letters::*;

    const TEST_INPUT: &str = r#"
MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX
    "#;

    fn build_example_grid() -> Grid {
        Grid(vec![
            vec![M, M, M, S, X, X, M, A, S, M],
            vec![M, S, A, M, X, M, S, M, S, A],
            vec![A, M, X, S, X, M, A, A, M, M],
            vec![M, S, A, M, A, S, M, S, M, X],
            vec![X, M, A, S, A, M, X, A, M, M],
            vec![X, X, A, M, M, X, X, A, M, A],
            vec![S, M, S, M, S, A, S, X, S, S],
            vec![S, A, X, A, M, A, S, A, A, A],
            vec![M, A, M, M, M, X, M, M, M, M],
            vec![M, X, M, X, A, X, M, A, S, X],
        ])
    }

    #[test]
    fn test_parse_grid() {
        let actual = TEST_INPUT.trim().parse::<Grid>().unwrap();
        let expected = build_example_grid();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_solve_part_1() {
        let grid = build_example_grid();
        let actual = solve_part_1(&grid);

        assert_eq!(actual, 18);
    }

    #[test]
    fn test_solve_part_2() {
        let grid = build_example_grid();
        let actual = solve_part_2(&grid);

        assert_eq!(actual, 9);
    }
}
