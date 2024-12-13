use anyhow::Result;
use regex::Regex;
use thiserror::Error;

const TRILLION: i64 = 10_000_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClawConfig {
    button_a: (i64, i64),
    button_b: (i64, i64),
    prize: (i64, i64),
}

impl ClawConfig {
    pub fn new(button_a: (i64, i64), button_b: (i64, i64), prize: (i64, i64)) -> Self {
        Self {
            button_a,
            button_b,
            prize,
        }
    }

    pub fn winnable(&self) -> bool {
        let gcd_x = gcd(self.button_a.0, self.button_b.0);
        let gcd_y = gcd(self.button_a.1, self.button_b.1);

        if self.prize.0 % gcd_x != 0 || self.prize.1 % gcd_y != 0 {
            false
        } else {
            true
        }
    }

    fn spider_hater_4_equation(&self) -> Option<(i64, i64)> {
        // Taken from this Reddit comment:
        // https://www.reddit.com/r/adventofcode/comments/1hd5b6o/comment/m1tx7yy/
        // `b=(py*ax-px*ay)/(by*ax-bx*ay) a=(px-b*bx)/ax`
        let (px, py) = self.prize;
        let (ax, ay) = self.button_a;
        let (bx, by) = self.button_b;

        let b = (py * ax - px * ay) / (by * ax - bx * ay);
        // Check if the division is exact (no remainder)
        if (py * ax - px * ay) % (by * ax - bx * ay) != 0 {
            return None;
        }

        let a = (px - b * bx) / ax;
        // Check if the division is exact (no remainder)
        if (px - b * bx) % ax != 0 {
            return None;
        }

        Some((a, b))
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        return a;
    }
    return gcd(b, a % b);
}

#[derive(Debug, Error)]
enum ClawConfigError {
    #[error("Invalid block")]
    InvalidBlock,
    #[error("Invalid number")]
    InvalidNumber(#[from] std::num::ParseIntError),
}

fn parse_input(input: &str) -> Result<Vec<ClawConfig>, ClawConfigError> {
    // Regex to match the three lines of each block
    // Written by CoPilot.
    let re = Regex::new(
        r"Button A: X\+(\d+), Y\+(\d+)\s+Button B: X\+(\d+), Y\+(\d+)\s+Prize: X=(\d+), Y=(\d+)",
    )
    .unwrap();

    let configs = input
        .replace("\r\n", "\n") // Normalize line endings
        .trim() // Remove leading/trailing whitespace
        .split("\n\n") // Split on blank lines
        .map(|block| {
            let caps = re.captures(block).ok_or(ClawConfigError::InvalidBlock)?;
            Ok(ClawConfig {
                button_a: (caps[1].parse()?, caps[2].parse()?),
                button_b: (caps[3].parse()?, caps[4].parse()?),
                prize: (caps[5].parse()?, caps[6].parse()?),
            })
        })
        .collect::<Result<Vec<_>, ClawConfigError>>()?;

    Ok(configs)
}

// Old solution, was passing tests but failed on the actual input.
// I'm keeping it here for reference. To see what I was doing wrong.
// I had to look up some help for this one. It wasn't obvious to me.
// https://en.wikipedia.org/wiki/Cramer%27s_rule
// fn cramer_rule(
//     dx_1: i64,
//     dy_1: i64,
//     dx_2: i64,
//     dy_2: i64,
//     t_x: i64,
//     t_y: i64,
// ) -> Option<(i64, i64)> {
//     let delta = dx_1 * dy_2 - dx_2 * dy_1;
//     if delta == 0 {
//         return None; // No unique solution
//     }

//     let delta_n1 = t_x * dy_2 - t_y * dx_2;
//     let delta_n2 = dx_1 * t_y - dy_1 * t_x;

//     let n1 = delta_n1 / delta;
//     let n2 = delta_n2 / delta;

//     if n1 < 0 || n2 < 0 {
//         return None;
//     }

//     Some((n1, n2))
// }

fn calculate_price(button_a: i64, button_b: i64) -> i64 {
    button_a * 3 + button_b
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;

    println!("Part 1: {}", part1(&input)?);
    println!("Part 2: {}", part2(&input)?);

    Ok(())
}

fn part1(input: &str) -> Result<i64> {
    let configs = parse_input(input)?;
    // Calculate the sum of the prices of the winnable games, using Spider Haters rule.
    let sum = configs
        .iter()
        .filter_map(|c| {
            if !c.winnable() {
                return None;
            }

            let (a, b) = c.spider_hater_4_equation()?;

            if a > 100 || b > 100 {
                None
            } else {
                if a < 0 || b < 0 {
                    println!("Negative values: a={}, b={}", a, b);
                }
                Some(calculate_price(a, b))
            }
        })
        .sum::<i64>();

    Ok(sum)
}

fn part2(input: &str) -> Result<i64> {
    let mut configs = parse_input(input)?;

    for c in &mut configs {
        c.prize.0 += TRILLION;
        c.prize.1 += TRILLION;
    }

    let sum = configs
        .iter()
        .filter_map(|c| {
            if !c.winnable() {
                return None;
            }

            let (a, b) = c.spider_hater_4_equation()?;

            if a < 0 || b < 0 {
                println!("Negative values: a={}, b={}", a, b);
            }
            Some(calculate_price(a, b))
        })
        .sum::<i64>();

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"
        Button A: X+94, Y+34
        Button B: X+22, Y+67
        Prize: X=8400, Y=5400

        Button A: X+26, Y+66
        Button B: X+67, Y+21
        Prize: X=12748, Y=12176

        Button A: X+17, Y+86
        Button B: X+84, Y+37
        Prize: X=7870, Y=6450

        Button A: X+69, Y+23
        Button B: X+27, Y+71
        Prize: X=18641, Y=10279
"#;

    fn create_test_configs() -> Vec<ClawConfig> {
        vec![
            ClawConfig::new((94, 34), (22, 67), (8400, 5400)),
            ClawConfig::new((26, 66), (67, 21), (12748, 12176)),
            ClawConfig::new((17, 86), (84, 37), (7870, 6450)),
            ClawConfig::new((69, 23), (27, 71), (18641, 10279)),
        ]
    }

    #[test]
    fn test_part2() {
        let expected = 0;
        let actual = part2(INPUT).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_input() {
        let expected = create_test_configs();
        let actual = parse_input(INPUT).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_cramer_rule() {
        let (dx_1, dy_1, dx_2, dy_2) = (94, 34, 22, 67);
        let (t_x, t_y) = (8400, 5400);
        let expected = Some((80, 40));
        let actual = cramer_rule(dx_1, dy_1, dx_2, dy_2, t_x, t_y);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_spider_hater() {
        let (ax, ay, bx, by) = (94, 34, 22, 67);
        let (px, py) = (8400, 5400);
        let expected = Some((80, 40));
        let actual = ClawConfig::new((ax, ay), (bx, by), (px, py)).spider_hater_4_equation();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calculate_price() {
        let expected = 280;
        let actual = calculate_price(80, 40);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_solve_part_1() {
        let expected = 480;
        let actual = part1(INPUT).unwrap();

        assert_eq!(actual, expected);
    }
}
