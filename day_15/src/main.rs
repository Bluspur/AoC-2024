use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use anyhow::Result;
use thiserror::Error;

/// Represents the main actor in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Robot(Point);

// Represents a 2d point in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Apply a movement to the point, returning the new point.
    pub fn apply_movement(self, movement: Movement) -> Self {
        match movement {
            Movement::Up => Self {
                x: self.x,
                y: self.y - 1,
            },
            Movement::Down => Self {
                x: self.x,
                y: self.y + 1,
            },
            Movement::Left => Self {
                x: self.x - 1,
                y: self.y,
            },
            Movement::Right => Self {
                x: self.x + 1,
                y: self.y,
            },
        }
    }

    /// Calculate the GPS score of the point.
    pub fn gps_coordinate_score(self) -> i32 {
        self.x + self.y * 100
    }
}

/// Represents a tile in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Wall,
    Empty,
    Box,
}

/// A graph representing a 2D grid of nodes.
/// Each node has a position and a tile type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    width: usize,
    height: usize,
    /// A map of points to node ids.
    nodes: HashMap<Point, usize>,
    /// A map of node ids to nodes.
    node_storage: HashMap<usize, Node>,
}

impl Graph {
    /// Creates a normalized representation of the graph, without respect to specific node ids.
    /// This is useful for comparing graphs for equality.
    pub fn normalize(&self) -> HashMap<Point, Node> {
        self.nodes
            .iter()
            .map(|(&point, &id)| (point, self.node_storage.get(&id).unwrap().clone()))
            .collect()
    }

    /// Takes a robot (starting point) and a set of instructions, and simulates the movement of the robot.
    /// Returns the final state of the graph and the robot.
    pub fn process_instructions(
        &self,
        robot: &Robot,
        instructions: &Instructions,
    ) -> (Graph, Robot) {
        let (mut graph, mut robot) = (self.clone(), *robot);
        for movement in &instructions.movements {
            graph.update(&mut robot, *movement);
        }

        (graph, robot)
    }

    /// Moves a box at the given position in the given direction.
    pub fn move_box(&mut self, box_pos: Point, direction: Movement) {
        let node_id = *self.nodes.get(&box_pos).expect("Node not found");
        let maybe_parts = self
            .node_storage
            .get(&node_id)
            .expect("Node not found")
            .parts;

        if let Some((old_left, old_right)) = maybe_parts {
            // If the node is a big box, then movement is more complex.
            match direction {
                Movement::Up | Movement::Down => {
                    // Both parts of the big box need to be moved.
                    let swap_left = old_left.apply_movement(direction);
                    let swap_right = old_right.apply_movement(direction);
                    // Get the node ids of the swapped positions.
                    let l_swap_id = *self.nodes.get(&swap_left).expect("Node not found");
                    let r_swap_id = *self.nodes.get(&swap_right).expect("Node not found");
                    // Update both swap nodes' inner neighbours.
                    let l_swap_node = self.node_storage.get_mut(&l_swap_id).unwrap();
                    l_swap_node.move_node(direction.opposite());
                    let r_swap_node = self.node_storage.get_mut(&r_swap_id).unwrap();
                    r_swap_node.move_node(direction.opposite());
                    // Update the box's inner neighbours.
                    let node = self.node_storage.get_mut(&node_id).unwrap();
                    node.move_node(direction);
                    // Swap the ids in the nodes map.
                    self.nodes.insert(old_left, l_swap_id);
                    self.nodes.insert(old_right, r_swap_id);
                    self.nodes.insert(swap_left, node_id);
                    self.nodes.insert(swap_right, node_id);
                }
                Movement::Left => {
                    // Only the right part of the big box needs to be moved.
                    let swap_pos = old_left.apply_movement(direction);
                    // Get the node id of swapped position
                    let swap_id = *self.nodes.get(&swap_pos).expect("Node not found");
                    // Update both nodes' inner neighbours.
                    // Move the swapped node in the opposite direction twice.
                    let swap_node = self.node_storage.get_mut(&swap_id).expect("Node not found");
                    swap_node.move_node(direction.opposite());
                    swap_node.move_node(direction.opposite());

                    let node = self.node_storage.get_mut(&node_id).unwrap();
                    node.move_node(direction);

                    // Swap the ids in the nodes map.
                    self.nodes.insert(old_right, swap_id);
                    self.nodes.insert(swap_pos, node_id);
                }
                Movement::Right => {
                    // Only the left part of the big box needs to be moved.
                    let swap_pos = old_right.apply_movement(direction);
                    // Get the node id of swapped position
                    let swap_id = *self.nodes.get(&swap_pos).expect("Node not found");
                    let swap_node = self.node_storage.get_mut(&swap_id).expect("Node not found");
                    // Update both nodes' inner neighbours.
                    // Move the swapped node in the opposite direction twice.
                    swap_node.move_node(direction.opposite());
                    swap_node.move_node(direction.opposite());

                    let node = self.node_storage.get_mut(&node_id).unwrap();
                    node.move_node(direction);
                    // Swap the ids in the nodes map.
                    self.nodes.insert(old_left, swap_id);
                    self.nodes.insert(swap_pos, node_id);
                }
            }
        } else {
            // If the node is not a big box, then movement is simple.
            let swap_pos = box_pos.apply_movement(direction);
            // Get the node id of swapped position
            let swap_id = *self.nodes.get(&swap_pos).expect("Node not found");
            let swap_node = self.node_storage.get_mut(&swap_id).expect("Node not found");
            swap_node.move_node(direction.opposite()); // Move the swapped node in the opposite direction.

            let node = self.node_storage.get_mut(&node_id).unwrap();
            node.move_node(direction);
            // Swap the ids in the nodes map.
            self.nodes.insert(box_pos, swap_id);
            self.nodes.insert(swap_pos, node_id);
        }
    }

    /// Updates the graph with the robot's intended movement for a single instruction.
    fn update(&mut self, robot: &mut Robot, movement: Movement) {
        let new_pos = robot.0.apply_movement(movement);
        // Initialize the search frontier with the robot's intended movement.
        let mut frontier = HashSet::from([new_pos]);
        // List of boxes to be moved, referenced by their points.
        let mut queue = vec![new_pos];
        let mut counter = 0;
        // We need to examine all nodes in the frontier together.
        while !frontier.is_empty() {
            counter += 1;
            if counter > 20 {
                break;
            }
            // Step 1. Convert the frontier into a vector of (point, node) tuples.
            let nodes = frontier
                .iter()
                // Map the frontier points to their node ids.
                .map(|point| (*self.nodes.get(point).unwrap(), *point))
                // Filter out any duplicate node ids.
                .collect::<HashMap<usize, Point>>()
                .iter()
                // Map the node ids to their nodes.
                .map(|(&id, &point)| (point, self.node_storage.get(&id).unwrap().clone()))
                // Collect the nodes into a vector.
                .collect::<Vec<_>>();

            // Step 2. Check if any of the nodes are walls.
            if nodes.iter().any(|(_, node)| node.tile == Tile::Wall) {
                return; // Skips the movement
            }

            // Step 3. Check if all nodes are empty.
            if nodes.iter().all(|(_, node)| node.tile == Tile::Empty) {
                break; // End the search
            }

            // Step 4. For any boxes, we need to add them to the queue and also add their neighbours to the frontier.
            let mut new_frontier = HashSet::new();
            for (pos, node) in nodes {
                if node.tile == Tile::Box {
                    // Add the box to the queue.
                    queue.push(pos);
                    // Add the neighbours of the box to the frontier.
                    new_frontier.extend(node.neighbours_in_direction(movement));
                }
            }

            frontier = new_frontier
        }

        // Move the robot to its new position.
        robot.0 = queue[0];
        queue.drain(0..1);
        // Step 5. If we have boxes to move, we need to move them.
        if !queue.is_empty() {
            // Reverse the queue and use a 2 item window to move the boxes.
            queue.reverse();
            for movable_box in queue.iter() {
                self.move_box(*movable_box, movement);
            }
        }
    }

    /// Calculate the total GPS score of every box in the graph.
    pub fn gps_scores(&self) -> usize {
        let mut scores = HashMap::new();
        // Get all the boxes and calculate their GPS scores.
        // For big boxes, use the score of the left part.
        // But use the lower score as the score for the box.
        self.nodes
            .iter()
            // We only care about boxes.
            .filter(|(_, id)| self.node_storage.get(id).unwrap().tile == Tile::Box)
            .for_each(|(point, id)| {
                let score = point.gps_coordinate_score() as usize;
                scores
                    .entry(id)
                    .and_modify(|e| {
                        if score < *e {
                            *e = score
                        }
                    })
                    .or_insert(score);
            });

        scores.values().sum()
    }

    /// Print the current state of the graph.
    pub fn print(&self, robot: Robot) {
        let mut big_box_open = false;
        for y in 0..self.height {
            for x in 0..self.width {
                let point = Point::new(x as i32, y as i32);
                let node_id = *self.nodes.get(&point).unwrap();
                let node = self.node_storage.get(&node_id).unwrap();

                if robot.0 == point {
                    print!("@");
                } else {
                    match node.tile {
                        Tile::Wall => print!("$"), // Used instead of '#' to avoid ligatures
                        Tile::Empty => print!("·"),
                        Tile::Box => {
                            if node.is_big_box() && !big_box_open {
                                print!("[")
                            } else if node.is_big_box() {
                                print!("]")
                            } else {
                                print!("O")
                            }
                        }
                    }
                }

                if node.is_big_box() {
                    big_box_open = !big_box_open;
                }
            }
            println!();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    tile: Tile,
    parts: Option<(Point, Point)>,
    n_neighbours: Vec<Point>,
    s_neighbours: Vec<Point>,
    e_neighbours: Vec<Point>,
    w_neighbours: Vec<Point>,
}

impl Node {
    pub fn new(pos: Point, tile: Tile) -> Self {
        Self {
            tile,
            parts: None,
            n_neighbours: vec![pos.apply_movement(Movement::Up)],
            s_neighbours: vec![pos.apply_movement(Movement::Down)],
            e_neighbours: vec![pos.apply_movement(Movement::Right)],
            w_neighbours: vec![pos.apply_movement(Movement::Left)],
        }
    }
    pub fn new_big_box(left: Point, right: Point) -> Self {
        Self {
            tile: Tile::Box,
            parts: Some((left, right)),
            n_neighbours: vec![
                left.apply_movement(Movement::Up),
                right.apply_movement(Movement::Up),
            ],
            s_neighbours: vec![
                left.apply_movement(Movement::Down),
                right.apply_movement(Movement::Down),
            ],
            e_neighbours: vec![right.apply_movement(Movement::Right)],
            w_neighbours: vec![left.apply_movement(Movement::Left)],
        }
    }
    pub fn move_node(&mut self, direction: Movement) {
        if let Some((left, right)) = self.parts {
            self.parts = Some((
                left.apply_movement(direction),
                right.apply_movement(direction),
            ));
        }
        self.n_neighbours.iter_mut().for_each(|n| {
            *n = n.apply_movement(direction);
        });
        self.s_neighbours.iter_mut().for_each(|n| {
            *n = n.apply_movement(direction);
        });
        self.e_neighbours.iter_mut().for_each(|n| {
            *n = n.apply_movement(direction);
        });
        self.w_neighbours.iter_mut().for_each(|n| {
            *n = n.apply_movement(direction);
        });
    }
    pub fn neighbours_in_direction(&self, direction: Movement) -> Vec<Point> {
        match direction {
            Movement::Up => self.n_neighbours.clone(),
            Movement::Down => self.s_neighbours.clone(),
            Movement::Left => self.w_neighbours.clone(),
            Movement::Right => self.e_neighbours.clone(),
        }
    }
    pub fn is_big_box(&self) -> bool {
        self.parts.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instructions {
    movements: Vec<Movement>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
}

impl Movement {
    pub fn opposite(&self) -> Self {
        match self {
            Movement::Up => Movement::Down,
            Movement::Down => Movement::Up,
            Movement::Left => Movement::Right,
            Movement::Right => Movement::Left,
        }
    }
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;

    let part_1 = part_1(&input)?;
    println!("Part 1: {}", part_1);

    let part_2 = part_2(&input)?;
    println!("Part 2: {}", part_2);

    Ok(())
}

fn part_1(input: &str) -> Result<usize> {
    let (map, instructions, robot) = parse_input(input, false)?;
    solve(map, instructions, robot)
}

fn part_2(input: &str) -> Result<usize> {
    let (map, instructions, robot) = parse_input(input, true)?;
    solve(map, instructions, robot)
}

fn solve(graph: Graph, instructions: Instructions, robot: Robot) -> Result<usize> {
    let (graph, _) = graph.process_instructions(&robot, &instructions);
    Ok(graph.gps_scores())
}

#[derive(Debug, Error)]
pub enum ParseInputError {
    #[error("Missing map or instructions")]
    MissingMapOrInstructions,
    #[error("Invalid map size: {0}x{1}")]
    InvalidMapSize(usize, usize),
    #[error("Invalid node count, expected: {0}, found: {1}")]
    InvalidNodeCount(usize, usize),
    #[error("Invalid character in map: {0}")]
    InvalidMapCharacter(char),
    #[error("Invalid character in instructions: '{0}'")]
    InvalidInstructionsCharacter(char),
    #[error("Robot not found in map")]
    RobotNotFound,
}

fn strip_whitespace_maintain_newlines(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_whitespace() || *c == '\n')
        .collect()
}

fn parse_input(input: &str, bigify: bool) -> Result<(Graph, Instructions, Robot), ParseInputError> {
    // Prepare the input by trimming whitespace and replacing Windows-style line endings with Unix-style line endings
    let input = input.trim().replace("\r\n", "\n");
    // Strip all whitespace except for newlines
    let input = strip_whitespace_maintain_newlines(&input);
    // Split the input into two parts: the map and the instructions (separated by a blank line)
    let (map_input, instructions_input) = input
        .split_once("\n\n")
        .ok_or(ParseInputError::MissingMapOrInstructions)?;

    let map_input = if bigify {
        biggify_map(map_input)
    } else {
        map_input.to_string()
    };

    let (graph, robot) = parse_graph(&map_input)?;
    let instructions = parse_instructions(instructions_input)?;

    Ok((graph, instructions, robot))
}

/// Required for part 2, simply doubles the size of the map in the horizontal direction.
fn biggify_map(input: &str) -> String {
    input
        .replace('#', "##")
        .replace("O", "[]")
        .replace(".", "..")
        .replace("@", "@.")
}

fn parse_graph(input: &str) -> Result<(Graph, Robot), ParseInputError> {
    let mut nodes = HashMap::new();
    let mut node_storage = HashMap::new();
    let mut robot = None;
    let mut id_counter = 0;

    let height = input.trim().lines().count();
    let width = input
        .trim()
        .lines()
        .map(|line| line.trim().chars().count())
        .max()
        .unwrap_or(0);

    if height == 0 || width == 0 {
        // Guard against empty maps
        return Err(ParseInputError::InvalidMapSize(width, height));
    }

    let mut node_counter = 0;

    for (y, line) in input.trim().lines().enumerate() {
        for (x, c) in line.trim().chars().enumerate() {
            if c.is_whitespace() {
                continue;
            }
            node_counter += 1;
            let point = Point::new(x as i32, y as i32);
            let node = match c {
                '#' => Ok(Node::new(point, Tile::Wall)),
                '.' => Ok(Node::new(point, Tile::Empty)),
                'O' => Ok(Node::new(point, Tile::Box)),
                '@' => {
                    // Robot is special, and always rests on an empty tile
                    robot = Some(Robot(point));
                    Ok(Node::new(point, Tile::Empty))
                }
                '[' => {
                    let left = point;
                    let right = point.apply_movement(Movement::Right);
                    Ok(Node::new_big_box(left, right))
                }
                // We can just add the current node to the map and continue
                ']' => {
                    nodes.insert(point, id_counter - 1);
                    continue;
                }
                _ => Err(ParseInputError::InvalidMapCharacter(c)),
            };

            let node = node?;
            nodes.insert(point, id_counter);
            node_storage.insert(id_counter, node);
            id_counter += 1;
        }
    }

    if nodes.len() != node_counter {
        return Err(ParseInputError::InvalidNodeCount(node_counter, nodes.len()));
    }

    let robot = robot.ok_or(ParseInputError::RobotNotFound)?;
    let graph = Graph {
        width,
        height,
        nodes,
        node_storage,
    };

    Ok((graph, robot))
}

fn parse_instructions(input: &str) -> Result<Instructions, ParseInputError> {
    let movements = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '^' => Ok(Movement::Up),
            'v' => Ok(Movement::Down),
            '<' => Ok(Movement::Left),
            '>' => Ok(Movement::Right),
            _ => Err(ParseInputError::InvalidInstructionsCharacter(c)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Instructions { movements })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL_INPUT: &str = r"
        ########
        #..O.O.#
        ##@.O..#
        #...O..#
        #.#.O..#
        #...O..#
        #......#
        ########

        <^^>>>vv<v>>v<<
    ";

    const LARGE_INPUT: &str = r"
        ##########
        #..O..O.O#
        #......O.#
        #.OO..O.O#
        #..O@..O.#
        #O#..O...#
        #O..O..O.#
        #.OO.O.OO#
        #....O...#
        ##########

        <vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
        vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
        ><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
        <<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
        ^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
        ^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
        >^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
        <><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
        ^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
        v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^
    ";

    const LARGE_INPUT_FINAL: &str = r"
        ##########
        #.O.O.OOO#
        #........#
        #OO......#
        #OO@.....#
        #O#.....O#
        #O.....OO#
        #O.....OO#
        #OO....OO#
        ##########
    ";

    const LARGE_INPUT_BIGGIFIED: &str = r"
        ####################
        ##....[]....[]..[]##
        ##............[]..##
        ##..[][]....[]..[]##
        ##....[]@.....[]..##
        ##[]##....[]......##
        ##[]....[]....[]..##
        ##..[][]..[]..[][]##
        ##........[]......##
        ####################

        <vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
        vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
        ><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
        <<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
        ^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
        ^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
        >^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
        <><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
        ^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
        v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^
    ";

    const LARGE_INPUT_BIGGIFIED_FINAL: &str = r"
        ####################
        ##[].......[].[][]##
        ##[]...........[].##
        ##[]........[][][]##
        ##[]......[]....[]##
        ##..##......[]....##
        ##..[]............##
        ##..@......[].[][]##
        ##......[][]..[]..##
        ####################
    ";

    // Helper macro to create a vector of movements from a pattern string.
    macro_rules! instructions {
        ($pattern:expr) => {{
            let mut movements = Vec::new();
            for ch in $pattern.chars() {
                let movement = match ch {
                    '<' => Some(Movement::Left),
                    '>' => Some(Movement::Right),
                    '^' => Some(Movement::Up),
                    'v' => Some(Movement::Down),
                    ' ' | '\n' | '\t' => None, // Ignore whitespace characters
                    _ => panic!("Unknown movement character"),
                };
                if let Some(movement) = movement {
                    movements.push(movement);
                }
            }
            Instructions { movements }
        }};
    }

    fn create_test_large_instructions() -> Instructions {
        instructions!(
            "<vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
            vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
            ><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
            <<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
            ^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
            ^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
            >^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
            <><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
            ^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
            v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^"
        )
    }

    #[test]
    fn test_calculate_gps_coordinate_score() {
        let point = Point::new(4, 1);
        assert_eq!(point.gps_coordinate_score(), 104);
    }

    #[test]
    fn test_calculate_simulate_large() {
        let (graph, _, robot) = parse_input(LARGE_INPUT, false).unwrap();
        let final_input = strip_whitespace_maintain_newlines(LARGE_INPUT_FINAL);
        let (graph_exp, robot_exp) = parse_graph(&final_input).unwrap();
        let instructions = create_test_large_instructions();

        let (graph, robot) = graph.process_instructions(&robot, &instructions);

        // Should only show on a fail
        graph.print(robot);
        println!();
        graph_exp.print(robot_exp);

        assert_eq!(graph.height, graph_exp.height);
        assert_eq!(graph.width, graph_exp.width);
        assert_eq!(graph.nodes.len(), graph_exp.nodes.len());
        assert_eq!(graph.node_storage.len(), graph_exp.node_storage.len());
        assert_eq!(robot, robot_exp);

        // Normalize the graphs and compare them. We aren't interested in the node ids.
        let (graph, graph_exp) = (graph.normalize(), graph_exp.normalize());
        assert_eq!(graph, graph_exp);
    }

    #[test]
    fn test_solve_part_1() {
        let expected = 2028;
        let actual = part_1(SMALL_INPUT).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bigification() {
        let (graph_exp, _, robot_exp) = parse_input(LARGE_INPUT_BIGGIFIED, false).unwrap();
        let (actual, _, robot) = parse_input(LARGE_INPUT, true).unwrap();

        assert_eq!(actual, graph_exp);
        assert_eq!(robot, robot_exp);
    }

    #[test]
    fn test_bigification_simulation() {
        let (graph, _, robot) = parse_input(LARGE_INPUT_BIGGIFIED, false).unwrap();
        let (graph_exp, robot_exp) = parse_graph(LARGE_INPUT_BIGGIFIED_FINAL).unwrap();
        let instructions = create_test_large_instructions();

        let (graph, robot) = graph.process_instructions(&robot, &instructions);

        // Should only show on a fail
        graph.print(robot);
        println!();
        graph_exp.print(robot_exp);

        assert_eq!(graph.height, graph_exp.height);
        assert_eq!(graph.width, graph_exp.width);
        assert_eq!(graph.nodes.len(), graph_exp.nodes.len());
        assert_eq!(graph.node_storage.len(), graph_exp.node_storage.len());
        assert_eq!(robot, robot_exp);

        // Normalize the graphs and compare them. We aren't interested in the node ids.
        let (graph, graph_exp) = (graph.normalize(), graph_exp.normalize());
        assert_eq!(graph, graph_exp);
    }

    #[test]
    #[ignore]
    fn test_solve_part_2() {
        let expected = 9021;
        let actual = part_2(LARGE_INPUT).unwrap();

        assert_eq!(actual, expected);
    }
}
