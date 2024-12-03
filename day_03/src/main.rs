use anyhow::{Context, Result};
use regex::Regex;

#[derive(Debug)]
enum Instruction {
    Mul(u32, u32),
    Do,
    Dont,
}

struct Mul {
    a: u32,
    b: u32,
}

impl Mul {
    fn resolve(&self) -> u32 {
        self.a * self.b
    }
}

fn main() -> Result<()> {
    let raw_input = std::fs::read_to_string("input.txt")?;

    let part_1_solution = solve_part_1(&raw_input)?;
    println!("Part 1 solution: {}", part_1_solution);

    let part_2_solution = solve_part_2(&raw_input)?;
    println!("Part 2 solution: {}", part_2_solution);

    Ok(())
}

fn solve_part_1(input: &str) -> Result<u32> {
    let regex = Regex::new(r"mul\((\d{1,3}),(\d{1,3})\)").context("Invalid regex")?;

    let mut instructions = Vec::new();

    for cap in regex.captures_iter(input) {
        let a = cap[1].parse()?;
        let b = cap[2].parse()?;

        instructions.push(Mul { a, b });
    }

    let sum = instructions.iter().map(|mul| mul.resolve()).sum();

    Ok(sum)
}

fn solve_part_2(input: &str) -> Result<u32> {
    let regex =
        Regex::new(r"mul\((\d{1,3}),(\d{1,3})\)|do\(\)|don't\(\)").context("Invalid regex")?;

    let mut instructions = Vec::new();

    for cap in regex.captures_iter(input) {
        if cap.get(1).is_some() {
            let a = cap[1].parse()?;
            let b = cap[2].parse()?;
            instructions.push(Instruction::Mul(a, b));
        } else if cap.get(0).unwrap().as_str() == "do()" {
            instructions.push(Instruction::Do);
        } else if cap.get(0).unwrap().as_str() == "don't()" {
            instructions.push(Instruction::Dont);
        }
    }

    let mut enabled = true;
    let mut sum = 0;

    for instruction in instructions {
        match instruction {
            Instruction::Do => enabled = true,
            Instruction::Dont => enabled = false,
            Instruction::Mul(a, b) if enabled => sum += a * b,
            _ => {}
        }
    }

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str =
        r#"xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))"#;
    const TEST_INPUT_2: &str =
        r#"xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))"#;

    #[test]
    fn test_part_1_solution() {
        let actual = solve_part_1(TEST_INPUT).unwrap();
        assert_eq!(actual, 161);
    }

    #[test]
    fn test_part_2_solution() {
        let actual = solve_part_2(TEST_INPUT_2).unwrap();
        assert_eq!(actual, 48);
    }
}
