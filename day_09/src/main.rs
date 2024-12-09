use std::num::ParseIntError;

use anyhow::Result;

struct DiskMap<T: DiskMapState> {
    state: T,
}

trait DiskMapState {}

struct Raw(String);
struct Parsed(Vec<usize>);
struct Expanded(Vec<Option<usize>>);
struct Compressed(Vec<usize>);

impl DiskMapState for Raw {}
impl DiskMapState for Parsed {}
impl DiskMapState for Expanded {}
impl DiskMapState for Compressed {}

/// Takes in a raw disk map and expands it using the block-empty-block pattern.
/// Wraps the block id in a Some() and the empty block in a None.
fn expand_disk_map(disk_map: Vec<usize>) -> Vec<Option<usize>> {
    let mut expanded_disk_map = Vec::new();
    let mut block_id = 0;
    for i in 0..disk_map.len() {
        let is_block = i % 2 == 0; // even
        let counter = disk_map[i];
        for _ in 0..counter {
            if is_block {
                expanded_disk_map.push(Some(block_id));
            } else {
                expanded_disk_map.push(None);
            }
        }
        if is_block {
            block_id += 1;
        }
    }

    expanded_disk_map
}

/// Takes in an expanded disk map and compresses it by moving the rigthmost block id to the leftmost
/// empty position. Finally it removes all empty blocks.
fn compress_disk_map(mut disk_map: Vec<Option<usize>>) -> Vec<usize> {
    // Iterate over the disk map in reverse order.
    // If the block is Some() then swap it with it with the leftmost empty block.
    for i in (0..disk_map.len()).rev() {
        if disk_map[i].is_some() {
            // Find the leftmost empty block.
            if let Some(empty_block) = disk_map[..i].iter().position(|block| block.is_none()) {
                // Swap the block id with the empty block.
                disk_map.swap(empty_block, i);
            }
        }
    }

    disk_map.into_iter().filter_map(|block| block).collect()
}

/// Calculates the checksum of the disk map by multiplying the block id with its new position.
fn calculate_checksum(disk_map: Vec<usize>) -> usize {
    disk_map
        .iter()
        .enumerate()
        .fold(0, |acc, (i, block)| acc + block * i)
}

/// Parses the raw input into a disk map.
fn parse_disk_map(raw_input: &str) -> Result<Vec<usize>, ParseIntError> {
    raw_input
        .trim()
        .chars()
        .map(|c| c.to_string().parse())
        .collect()
}

fn main() -> Result<()> {
    let raw_input = std::fs::read_to_string("input.txt")?;
    let disk_map = parse_disk_map(&raw_input)?;

    // Part 1
    let part_1 = solve_part_1(disk_map);
    println!("Part 1: {}", part_1);

    Ok(())
}

fn solve_part_1(disk_map: Vec<usize>) -> usize {
    let expanded_disk_map = expand_disk_map(disk_map);
    let compressed_disk_map = compress_disk_map(expanded_disk_map);
    let checksum = calculate_checksum(compressed_disk_map);

    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "2333133121414131402";

    fn create_test_disk_map() -> Vec<usize> {
        vec![2, 3, 3, 3, 1, 3, 3, 1, 2, 1, 4, 1, 4, 1, 3, 1, 4, 0, 2]
    }

    #[test]
    fn test_parse_input() {
        let expected = create_test_disk_map();
        let actual = parse_disk_map(TEST_INPUT).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_expand_disk_map() {
        let disk_map = create_test_disk_map();
        let expected = vec![
            Some(0),
            Some(0),
            None,
            None,
            None,
            Some(1),
            Some(1),
            Some(1),
            None,
            None,
            None,
            Some(2),
            None,
            None,
            None,
            Some(3),
            Some(3),
            Some(3),
            None,
            Some(4),
            Some(4),
            None,
            Some(5),
            Some(5),
            Some(5),
            Some(5),
            None,
            Some(6),
            Some(6),
            Some(6),
            Some(6),
            None,
            Some(7),
            Some(7),
            Some(7),
            None,
            Some(8),
            Some(8),
            Some(8),
            Some(8),
            Some(9),
            Some(9),
        ];
        let actual = expand_disk_map(disk_map);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_compress_disk_map() {
        let disk_map = create_test_disk_map();
        let expanded_disk_map = expand_disk_map(disk_map);
        let expected = vec![
            0, 0, 9, 9, 8, 1, 1, 1, 8, 8, 8, 2, 7, 7, 7, 3, 3, 3, 6, 4, 4, 6, 5, 5, 5, 5, 6, 6,
        ];
        let actual = compress_disk_map(expanded_disk_map);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_solve_part_1() {
        let disk_map = parse_disk_map(TEST_INPUT).unwrap();

        let expected = 1928;
        let actual = solve_part_1(disk_map);

        assert_eq!(expected, actual);
    }
}
