use anyhow::Result;
use thiserror::Error;

#[derive(Debug, PartialEq)]
struct Equation {
    outcome: u64,
    values: Vec<u64>,
}

impl Equation {
    fn validate(&self) -> bool {
        evaluate(&self.values, self.outcome, 0)
    }

    fn validate_with_concatenate(&self) -> bool {
        evaluate_with_concatenate(&self.values, self.outcome, 0)
    }
}

/// Evaluate the given values to see if they can be combined to reach the target
/// This is a recursive function that will try to add or multiply the first value
/// with the result of evaluating the rest of the values
fn evaluate(values: &[u64], target: u64, current: u64) -> bool {
    let Some((first, rest)) = values.split_first() else {
        return current == target;
    };
    evaluate(rest, target, current + first) || evaluate(rest, target, current * first)
}

/// Evaluate the given values to see if they can be combined to reach the target
/// This is a recursive function that will try to add, multiply, or concatenate the first value
/// with the result of evaluating the rest of the values
fn evaluate_with_concatenate(values: &[u64], target: u64, current: u64) -> bool {
    let Some((first, rest)) = values.split_first() else {
        return current == target;
    };
    evaluate_with_concatenate(rest, target, current + first)
        || evaluate_with_concatenate(rest, target, current * first)
        || evaluate_with_concatenate(rest, target, concatenate(current, *first))
}

fn concatenate(a: u64, b: u64) -> u64 {
    // We need to copy b because we need the original value later
    let mut b_cpy = b;
    let mut shifted = 1;

    // Shift the number to the left by the number of digits in b
    // b == 0 is a special case
    if b_cpy == 0 {
        shifted = 10;
    } else {
        while b_cpy > 0 {
            shifted *= 10;
            b_cpy /= 10;
        }
    }

    // Multiply a by the shifted value and add b
    a * shifted + b
}

#[derive(Debug, Error)]
enum ParseError {
    #[error("Failed to parse integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Malformed input")]
    Malformed,
}

fn parse_input(input: &str) -> Result<Vec<Equation>, ParseError> {
    // Always need to trim the input to remove leading/trailing whitespace
    // It's kind of annoying that we have to do this in multiple places
    input
        .trim()
        .lines()
        .map(|line| {
            // Split the line into two parts, separated by ": "
            let (outcome, values) = line.trim().split_once(": ").ok_or(ParseError::Malformed)?;
            // Parse the outcome as a u64
            let outcome = outcome.parse()?;
            // Split the values by spaces, parse each value as a u64, and collect them into a Vec
            let values = values
                .split(' ')
                .map(str::parse)
                .collect::<Result<Vec<_>, _>>()?;
            // Closure needs to return a Result to allow the ? operator to be used, so wrap in Ok
            Ok(Equation { outcome, values })
        })
        // Handles the Result from the closure, returning the Vec<Equation> or the error
        .collect()
}

fn main() -> Result<()> {
    let raw_input = std::fs::read_to_string("input.txt")?;
    let equations = parse_input(&raw_input)?;

    // Part 1
    let part_1 = part_1(&equations);
    println!("Part 1: {}", part_1);

    // Part 2
    let part_2 = part_2(&equations);
    println!("Part 2: {}", part_2);

    Ok(())
}

fn part_1(equations: &[Equation]) -> u64 {
    equations
        .iter()
        .filter(|eq| eq.validate())
        .map(|eq| eq.outcome)
        .sum()
}

fn part_2(equations: &[Equation]) -> u64 {
    equations
        .iter()
        .filter(|eq| eq.validate_with_concatenate())
        .map(|eq| eq.outcome)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Whitespace needs to be stripped from each line
    const TEST_INPUT: &str = r#"
        190: 10 19
        3267: 81 40 27
        83: 17 5
        156: 15 6
        7290: 6 8 6 15
        161011: 16 10 13
        192: 17 8 14
        21037: 9 7 18 13
        292: 11 6 16 20
    "#;

    fn test_equations() -> Vec<Equation> {
        vec![
            Equation {
                outcome: 190,
                values: vec![10, 19],
            },
            Equation {
                outcome: 3267,
                values: vec![81, 40, 27],
            },
            Equation {
                outcome: 83,
                values: vec![17, 5],
            },
            Equation {
                outcome: 156,
                values: vec![15, 6],
            },
            Equation {
                outcome: 7290,
                values: vec![6, 8, 6, 15],
            },
            Equation {
                outcome: 161011,
                values: vec![16, 10, 13],
            },
            Equation {
                outcome: 192,
                values: vec![17, 8, 14],
            },
            Equation {
                outcome: 21037,
                values: vec![9, 7, 18, 13],
            },
            Equation {
                outcome: 292,
                values: vec![11, 6, 16, 20],
            },
        ]
    }

    #[test]
    fn test_parse_input() {
        let equations = parse_input(TEST_INPUT).unwrap();
        assert_eq!(equations, test_equations());
    }

    fn test_known_equations() -> Vec<Equation> {
        let mut equations = Vec::new();
        equations.push(Equation {
            outcome: 190,
            values: vec![10, 19],
        });
        equations.push(Equation {
            outcome: 3267,
            values: vec![81, 40, 27],
        });
        equations.push(Equation {
            outcome: 292,
            values: vec![11, 6, 16, 20],
        });
        equations
    }

    #[test]
    fn test_validate_equations() {
        let equations = test_known_equations();

        for equation in equations.iter() {
            assert!(equation.validate());
        }
    }

    #[test]
    fn test_concatenate() {
        assert_eq!(concatenate(123, 456), 123456);
        assert_eq!(concatenate(0, 456), 456);
        assert_eq!(concatenate(123, 0), 1230);
    }

    #[test]
    fn test_evaluate_with_concatenate() {
        let known_eqs = test_known_equations();
        let known_concats = vec![
            Equation {
                outcome: 156,
                values: vec![15, 6],
            },
            Equation {
                outcome: 7290,
                values: vec![6, 8, 6, 15],
            },
            Equation {
                outcome: 192,
                values: vec![17, 8, 14],
            },
        ];

        // First check that the known equations pass
        for equation in known_eqs.iter() {
            assert!(equation.validate_with_concatenate());
        }
        // Then check if the known concatenations pass
        for equation in known_concats.iter() {
            assert!(equation.validate_with_concatenate());
        }
    }
}
