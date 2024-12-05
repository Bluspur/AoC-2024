use std::{
    collections::{HashMap, HashSet, VecDeque},
    str::FromStr,
};

use anyhow::Result;
use thiserror::Error;

type Page = u32;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OrderingRule {
    value: Page,
    before: Page,
}

impl OrderingRule {
    pub fn new(value: Page, before: Page) -> Self {
        Self { value, before }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Update(Vec<Page>);

impl Update {
    fn is_valid(&self, rules: &[OrderingRule]) -> bool {
        // Create a map of pages to their index in the update.
        let map = self.0.iter().enumerate().fold(
            std::collections::HashMap::new(),
            |mut map, (i, &page)| {
                map.insert(page, i);
                map
            },
        );

        for rule in rules {
            if let (Some(&before), Some(&after)) = (map.get(&rule.value), map.get(&rule.before)) {
                if before > after {
                    return false;
                }
            }
        }

        true
    }

    // Don't use this lol, it took over 15 minutes to run on the input.
    // I'm keeping it here for posterity.
    fn _correct_update_brute_force(&self, rules: &[OrderingRule]) -> Update {
        let map = self.0.iter().enumerate().fold(
            std::collections::HashMap::new(),
            |mut map, (i, &page)| {
                map.insert(page, i);
                map
            },
        );

        // Filter rules to only those which are relevant to the update.
        let relevant_rules = rules
            .iter()
            .filter(|rule| map.contains_key(&rule.value) && map.contains_key(&rule.before))
            .copied()
            .collect::<Vec<OrderingRule>>();

        // Somehow I need to take the Update and apply all the rules to it, so that it is sorted correctly.
        // An issue is that a single run through the rules may not be enough to sort the update correctly.
        // Brute force is an option, but it's not a good one.

        let mut update = self.clone();

        while !update.is_valid(&relevant_rules) {
            for rule in &relevant_rules {
                if let (Some(&before), Some(&after)) = (map.get(&rule.value), map.get(&rule.before))
                {
                    if before > after {
                        // Swap the two pages.
                        let before_index = map[&rule.before];
                        let after_index = map[&rule.value];
                        update.0.swap(before_index, after_index);
                    }
                }
            }
        }

        update
    }

    // Mr CoPilot pointed me in the right direction with this one.
    // I've added some comments for my own understanding.
    fn correct_update(&self, rules: &[OrderingRule]) -> Update {
        let mut graph: HashMap<Page, HashSet<Page>> = HashMap::new();
        let mut in_degree: HashMap<Page, usize> = HashMap::new();

        // Initialize the graph and in-degree map
        for &page in &self.0 {
            graph.entry(page).or_default();
            in_degree.entry(page).or_insert(0);
        }

        // Build the graph and in-degree map based on the rules
        for rule in rules {
            // Skip rules which are not relevant to the update. Doesn't work without this check.
            if !graph.contains_key(&rule.value) || !graph.contains_key(&rule.before) {
                continue;
            }

            if let Some(neighbors) = graph.get_mut(&rule.value) {
                if neighbors.insert(rule.before) {
                    *in_degree.entry(rule.before).or_insert(0) += 1;
                }
            }
        }

        // Perform topological sort
        let mut queue: VecDeque<Page> = VecDeque::new();
        for (&page, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(page);
            }
        }

        let mut sorted_pages = Vec::new();
        while let Some(page) = queue.pop_front() {
            sorted_pages.push(page);
            if let Some(neighbors) = graph.get(&page) {
                for &neighbor in neighbors {
                    let degree = in_degree.get_mut(&neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // If the sorted_pages length is not equal to the original update length, it means there was a cycle
        if sorted_pages.len() != self.0.len() {
            panic!("Cycle detected in the rules");
        }

        Update(sorted_pages)
    }

    fn get_middle_page(&self) -> Page {
        // Assuming the update is valid. I.e. has an odd number of pages.
        self.0[self.0.len() / 2]
    }
}

#[derive(Debug, PartialEq)]
struct PrintQueue {
    rules: Vec<OrderingRule>,
    updates: Vec<Update>,
}

impl PrintQueue {
    fn get_valid_updates(&self) -> Vec<&Update> {
        self.updates
            .iter()
            .filter(|update| update.is_valid(&self.rules))
            .collect()
    }

    fn get_invalid_updates(&self) -> Vec<&Update> {
        self.updates
            .iter()
            .filter(|update| !update.is_valid(&self.rules))
            .collect()
    }
}

#[derive(Debug, Error)]
enum PrintQueueError {
    #[error("Invalid page number: {0}")]
    CannotParseInt(#[from] std::num::ParseIntError),
    #[error("Invalid queue")]
    MalformedQueue,
    #[error("Invalid rule: {0}")]
    MalformedRule(String),
    #[error("Invalid update: {0}")]
    MalformedUpdate(String),
}

impl FromStr for PrintQueue {
    type Err = PrintQueueError;

    // Works well enough. A possible optimization would be to only include rules which are included in an update.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rules = Vec::new();
        let mut updates = Vec::new();

        // Normalize newlines to avoid the normal annoying crap.
        let s = s.replace("\r\n", "\n");

        let (rules_str, updates_str) = s
            .split_once("\n\n")
            .ok_or(PrintQueueError::MalformedQueue)?;

        for line in rules_str.trim().lines() {
            let (value_str, before_str) = line
                .split_once('|')
                .ok_or(PrintQueueError::MalformedRule(line.to_string()))?;
            let value = value_str.parse()?;
            let before = before_str.parse()?;
            rules.push(OrderingRule { value, before });
        }

        for line in updates_str.trim().lines() {
            let update = line
                .split(',')
                .map(|s| s.parse())
                .collect::<Result<Vec<Page>, _>>()?;
            if update.len() % 2 == 0 {
                return Err(PrintQueueError::MalformedUpdate(line.to_string()));
            }
            updates.push(Update(update));
        }

        Ok(PrintQueue { rules, updates })
    }
}

fn main() -> Result<()> {
    let raw_input = std::fs::read_to_string("input.txt")?;
    let print_queue: PrintQueue = raw_input.parse()?;

    // Part 1
    let part_1 = solve_part_1(&print_queue);
    println!("Part 1: {}", part_1);

    // Part 2
    let part_2 = solve_part_2(&print_queue);
    println!("Part 2: {}", part_2);

    Ok(())
}

fn solve_part_1(print_queue: &PrintQueue) -> u32 {
    let valid_updates = print_queue.get_valid_updates();
    valid_updates
        .iter()
        .map(|update| update.get_middle_page())
        .sum()
}

fn solve_part_2(print_queue: &PrintQueue) -> u32 {
    let invalid_updates = print_queue.get_invalid_updates();
    invalid_updates
        .iter()
        .map(|update| update.correct_update(&print_queue.rules))
        .map(|update| update.get_middle_page())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STRING: &str = r#"
47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47
"#;

    fn create_test_print_queue() -> PrintQueue {
        PrintQueue {
            rules: vec![
                OrderingRule::new(47, 53),
                OrderingRule::new(97, 13),
                OrderingRule::new(97, 61),
                OrderingRule::new(97, 47),
                OrderingRule::new(75, 29),
                OrderingRule::new(61, 13),
                OrderingRule::new(75, 53),
                OrderingRule::new(29, 13),
                OrderingRule::new(97, 29),
                OrderingRule::new(53, 29),
                OrderingRule::new(61, 53),
                OrderingRule::new(97, 53),
                OrderingRule::new(61, 29),
                OrderingRule::new(47, 13),
                OrderingRule::new(75, 47),
                OrderingRule::new(97, 75),
                OrderingRule::new(47, 61),
                OrderingRule::new(75, 61),
                OrderingRule::new(47, 29),
                OrderingRule::new(75, 13),
                OrderingRule::new(53, 13),
            ],
            updates: vec![
                Update(vec![75, 47, 61, 53, 29]),
                Update(vec![97, 61, 53, 29, 13]),
                Update(vec![75, 29, 13]),
                Update(vec![75, 97, 47, 61, 53]),
                Update(vec![61, 13, 29]),
                Update(vec![97, 13, 75, 29, 47]),
            ],
        }
    }

    #[test]
    fn test_parse() {
        let actual: PrintQueue = TEST_STRING.parse().unwrap();
        let expected = create_test_print_queue();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_update_is_valid() {
        let print_queue = create_test_print_queue();

        assert!(print_queue.updates[0].is_valid(&print_queue.rules));
        assert!(print_queue.updates[1].is_valid(&print_queue.rules));
        assert!(print_queue.updates[2].is_valid(&print_queue.rules));
        assert!(!print_queue.updates[3].is_valid(&print_queue.rules));
        assert!(!print_queue.updates[4].is_valid(&print_queue.rules));
        assert!(!print_queue.updates[5].is_valid(&print_queue.rules));
    }

    #[test]
    fn test_solve_part_1() {
        let print_queue: PrintQueue = TEST_STRING.parse().unwrap();
        let actual = solve_part_1(&print_queue);

        assert_eq!(143, actual);
    }

    #[test]
    fn test_solve_part_2() {
        let print_queue: PrintQueue = TEST_STRING.parse().unwrap();
        let actual = solve_part_2(&print_queue);

        assert_eq!(123, actual);
    }
}
