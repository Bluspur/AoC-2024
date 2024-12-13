use std::collections::HashMap;

use anyhow::Result;

#[derive(Debug, PartialEq, Clone)]
struct Stones(Vec<usize>);

impl Stones {
    fn engrave(&mut self) -> &mut Self {
        let mut result = Vec::new();

        for stone in self.0.iter() {
            let rule = Rule::find(*stone);
            match rule {
                Rule::Flip => result.push(1),
                Rule::Split => {
                    let (a, b) = split_integer(*stone);
                    result.push(a);
                    result.push(b);
                }
                Rule::Multiply => result.push(*stone * 2024),
            }
        }

        self.0 = result;
        self
    }

    fn engrave_n_times(&mut self, n: usize) -> &mut Self {
        for _ in 0..n {
            self.engrave();
        }
        self
    }

    fn count_stones(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, PartialEq)]
enum Rule {
    Flip,
    Split,
    Multiply,
}

impl Rule {
    fn find(n: usize) -> Rule {
        if n == 0 {
            Rule::Flip
        } else if count_digits(n) % 2 == 0 {
            Rule::Split
        } else {
            Rule::Multiply
        }
    }
}

/// Assumes that the input is a positive integer
/// and has an even number of digits.
fn split_integer(n: usize) -> (usize, usize) {
    let half_digits = count_digits(n) / 2;
    let divisor = 10usize.pow(half_digits as u32);
    (n / divisor, n % divisor)
}

/// Counts the number of digits in a positive integer.
fn count_digits(n: usize) -> usize {
    let mut counter = 0;
    let mut num = n;
    while num > 0 {
        num /= 10;
        counter += 1;
    }
    counter
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;
    let stones = Stones(
        input
            .trim()
            .split_ascii_whitespace()
            .map(|s| s.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()?,
    );

    // Part 1
    let part_1 = solve_part_1(stones.clone());
    println!("Part 1: {}", part_1);

    // Part 2
    let part_2 = solve_part_2(stones);
    println!("Part 2: {}", part_2);

    Ok(())
}

fn solve_part_1(mut stones: Stones) -> usize {
    stones.engrave_n_times(25);
    stones.count_stones()
}

/// This function is pretty much a 1 to 1 copy of the solution from the following
/// YouTube video: https://www.youtube.com/watch?v=La6OcNBUjVo
/// I had no idea how to solve the second part of the problem, so I looked up a solution.
fn solve_part_2(stones: Stones) -> usize {
    // Cast the stones into a HashMap since the order does not actually matter.
    let mut current = stones
        .0
        .iter()
        .map(|&x| (x, 1))
        .collect::<HashMap<usize, usize>>();

    // Engrave the stones 75 times.
    for _ in 0..75 {
        // Create a new HashMap to store the next iteration of stones.
        let mut next = HashMap::new();
        // Iterate over the current stones.
        for (stone, count) in current {
            // Split the stone into either 1 or 2 new stones, depending on the rule.
            for new_stone in split_stone(stone) {
                // Insert the new stone into the next HashMap.
                let entry = next.entry(new_stone).or_default();
                // Add the count of the current stone to the new stone.
                *entry += count;
            }
        }

        // Set the current HashMap to the next HashMap.
        current = next;
    }

    current.values().sum()
}

fn split_stone(stone: usize) -> Vec<usize> {
    if stone == 0 {
        vec![1]
    } else if count_digits(stone) % 2 == 0 {
        let (a, b) = split_integer(stone);
        vec![a, b]
    } else {
        vec![stone * 2024]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_integer() {
        assert_eq!(split_integer(1234), (12, 34));
        assert_eq!(split_integer(123456), (123, 456));
        assert_eq!(split_integer(12345678), (1234, 5678));
        assert_eq!(split_integer(99), (9, 9));
    }

    #[test]
    fn test_count_digits() {
        assert_eq!(count_digits(1234), 4);
        assert_eq!(count_digits(123456), 6);
        assert_eq!(count_digits(12345678), 8);
    }

    #[test]
    fn test_engrave_stones() {
        let mut stones = Stones(vec![0, 1, 10, 99, 999]);
        let expected = Stones(vec![1, 2024, 1, 0, 9, 9, 2021976]);
        stones.engrave();
        assert_eq!(expected, stones);
    }

    #[test]
    fn test_engrave_stones_repeated() {
        let mut stones = Stones(vec![125, 17]);

        stones.engrave();
        assert_eq!(vec![253000, 1, 7], stones.0);
        stones.engrave();
        assert_eq!(vec![253, 0, 2024, 14168], stones.0);
        stones.engrave();
        assert_eq!(vec![512072, 1, 20, 24, 28676032], stones.0);
        stones.engrave();
        assert_eq!(vec![512, 72, 2024, 2, 0, 2, 4, 2867, 6032], stones.0);
        stones.engrave();
        assert_eq!(
            vec![1036288, 7, 2, 20, 24, 4048, 1, 4048, 8096, 28, 67, 60, 32],
            stones.0
        );
        stones.engrave();
        assert_eq!(
            Stones(vec![
                2097446912, 14168, 4048, 2, 0, 2, 4, 40, 48, 2024, 40, 48, 80, 96, 2, 8, 6, 7, 6,
                0, 3, 2
            ]),
            stones
        );
    }

    #[test]
    fn test_engrave_stones_n_times() {
        let mut stones = Stones(vec![125, 17]);

        stones.engrave_n_times(25);

        assert_eq!(55312, stones.count_stones());
    }

    #[test]
    fn test_find_rules() {
        assert_eq!(Rule::find(0), Rule::Flip);
        assert_eq!(Rule::find(1), Rule::Multiply);
        assert_eq!(Rule::find(10), Rule::Split);
    }
}
