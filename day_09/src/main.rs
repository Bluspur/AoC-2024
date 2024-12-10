use std::collections::HashSet;

use anyhow::Result;

#[derive(Debug, PartialEq)]
struct DiskMap<T: DiskMapState> {
    state: T,
}

trait DiskMapState {}

#[derive(Debug)]
struct Raw(String);
#[derive(Debug, PartialEq)]
struct Parsed(Vec<usize>);
#[derive(Debug, PartialEq)]
struct Expanded(Vec<Option<usize>>);
#[derive(Debug, PartialEq)]
struct Compressed(Vec<Option<usize>>);

impl DiskMapState for Raw {}
impl DiskMapState for Parsed {}
impl DiskMapState for Expanded {}
impl DiskMapState for Compressed {}

impl DiskMap<Raw> {
    fn new(input: String) -> DiskMap<Raw> {
        DiskMap { state: Raw(input) }
    }
    fn parse(self) -> Result<DiskMap<Parsed>> {
        let inner = self
            .state
            .0
            .trim()
            .chars()
            .map(|c| c.to_string().parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DiskMap {
            state: Parsed(inner),
        })
    }
}

impl DiskMap<Parsed> {
    /// Takes in a raw disk map and expands it using the block-empty-block pattern.
    /// Wraps the block id in a Some() and the empty block in a None.
    fn expand(self) -> DiskMap<Expanded> {
        let mut expanded_disk_map = Vec::new();
        let mut block_id = 0;
        for i in 0..self.state.0.len() {
            let is_block = i % 2 == 0; // even
            let counter = self.state.0[i];
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

        DiskMap {
            state: Expanded(expanded_disk_map),
        }
    }
}

impl DiskMap<Expanded> {
    /// Takes in an expanded disk map and compresses it by moving the rigthmost block id to the leftmost
    /// empty position.
    fn compress(self) -> DiskMap<Compressed> {
        let mut disk_map = self.state.0;
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

        DiskMap {
            state: Compressed(disk_map),
        }
    }

    /// Takes in an expanded disk map and compresses it by moving whole files to the leftmost empty position.
    /// Differs from compress() by moving whole files instead of single blocks. May result in empty blocks.
    fn compress_continguous(self) -> DiskMap<Compressed> {
        let disk_map = self.state.0;

        // Internally compress the disk map into a vector of tuples where the first element is the Opton<usize>
        // and the second element is the number of contiguous blocks.
        let compressed_disk_map = disk_map.iter().fold(Vec::new(), |mut acc, block| {
            if let Some((last_block, count)) = acc.last_mut() {
                if *last_block == block {
                    *count += 1;
                } else {
                    acc.push((block, 1));
                }
            } else {
                acc.push((block, 1));
            }
            acc
        });

        let mut final_compressed_disk_map = Vec::new();
        let mut explored = HashSet::new();
        for (block, len) in &compressed_disk_map {
            match block {
                Some(v) => {
                    if !explored.contains(v) {
                        explored.insert(*v);
                        for _ in 0..*len {
                            final_compressed_disk_map.push(Some(*v));
                        }
                    } else {
                        // If the block id is already explored then it is an empty block.
                        for _ in 0..*len {
                            final_compressed_disk_map.push(None);
                        }
                    }
                }
                None => {
                    let mut len = *len;

                    while len > 0 {
                        // Find the last block id that would fit in the empty block.
                        if let Some((mvd_id, mvd_len)) =
                            compressed_disk_map.iter().rev().find(|(id, v)| {
                                // Check if the block id is not already explored and the block id is less than or equal to the length.
                                if let Some(id) = id {
                                    !explored.contains(id) && *v <= len
                                } else {
                                    // If the block id is None then it is an empty block and should be ignored.
                                    false
                                }
                            })
                        {
                            // Mark the block id as explored.
                            explored.insert(mvd_id.unwrap());
                            // Add the block id to the compressed disk map.
                            for _ in 0..*mvd_len {
                                final_compressed_disk_map.push(**mvd_id);
                            }
                            // Subtract the length of the block id from the empty block.
                            len -= mvd_len;
                        } else {
                            // No block id was found that would fit in the empty block.
                            for _ in 0..len {
                                final_compressed_disk_map.push(None);
                            }
                            break;
                        }
                    }
                }
            }
        }

        DiskMap {
            state: Compressed(final_compressed_disk_map),
        }
    }
}

impl DiskMap<Compressed> {
    /// Calculates the checksum of the disk map by multiplying the block id with its new position.
    fn checksum(&self) -> usize {
        self.state.0.iter().enumerate().fold(0, |acc, (i, block)| {
            if let Some(value) = block {
                acc + value * i
            } else {
                acc
            }
        })
    }
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;

    // Part 1
    let part_1 = solve_part_1(&input)?;
    println!("Part 1: {}", part_1);

    // Part 2
    let part_2 = solve_part_2(&input)?;
    println!("Part 2: {}", part_2);

    Ok(())
}

fn solve_part_1(input: &str) -> Result<usize> {
    Ok(DiskMap::new(input.to_string())
        .parse()?
        .expand()
        .compress()
        .checksum())
}

fn solve_part_2(input: &str) -> Result<usize> {
    Ok(DiskMap::new(input.to_string())
        .parse()?
        .expand()
        .compress_continguous()
        .checksum())
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "2333133121414131402";

    fn create_raw_disk_map() -> DiskMap<Raw> {
        DiskMap {
            state: Raw(INPUT.to_string()),
        }
    }

    fn create_parsed_disk_map() -> DiskMap<Parsed> {
        DiskMap {
            state: Parsed(vec![
                2, 3, 3, 3, 1, 3, 3, 1, 2, 1, 4, 1, 4, 1, 3, 1, 4, 0, 2,
            ]),
        }
    }

    fn create_expanded_disk_map() -> DiskMap<Expanded> {
        DiskMap {
            state: Expanded(vec![
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
            ]),
        }
    }

    fn create_compressed_disk_map() -> DiskMap<Compressed> {
        DiskMap {
            state: Compressed(vec![
                Some(0),
                Some(0),
                Some(9),
                Some(9),
                Some(8),
                Some(1),
                Some(1),
                Some(1),
                Some(8),
                Some(8),
                Some(8),
                Some(2),
                Some(7),
                Some(7),
                Some(7),
                Some(3),
                Some(3),
                Some(3),
                Some(6),
                Some(4),
                Some(4),
                Some(6),
                Some(5),
                Some(5),
                Some(5),
                Some(5),
                Some(6),
                Some(6),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]),
        }
    }

    fn create_compressed_contiguous_disk_map() -> DiskMap<Compressed> {
        DiskMap {
            state: Compressed(vec![
                Some(0),
                Some(0),
                Some(9),
                Some(9),
                Some(2),
                Some(1),
                Some(1),
                Some(1),
                Some(7),
                Some(7),
                Some(7),
                None,
                Some(4),
                Some(4),
                None,
                Some(3),
                Some(3),
                Some(3),
                None,
                None,
                None,
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
                None,
                None,
                None,
                None,
                Some(8),
                Some(8),
                Some(8),
                Some(8),
                None,
                None,
            ]),
        }
    }

    #[test]
    fn test_parse_input() {
        let expected = create_parsed_disk_map();
        let actual = create_raw_disk_map().parse().unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_expand_disk_map() {
        let expected = create_expanded_disk_map();
        let actual = create_parsed_disk_map().expand();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_compress_disk_map() {
        let expected = create_compressed_disk_map();
        let actual = create_expanded_disk_map().compress();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_compress_continguous_disk_map() {
        let expected = create_compressed_contiguous_disk_map();
        let actual = create_expanded_disk_map().compress_continguous();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_solve_part_1() {
        let expected = 1928;
        let actual = solve_part_1(INPUT).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_solve_part_2() {
        let expected = 2858;
        let actual = solve_part_2(INPUT).unwrap();

        assert_eq!(expected, actual);
    }
}
