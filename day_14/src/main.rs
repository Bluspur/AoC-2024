use std::{
    collections::{HashMap, HashSet},
    ops::Add,
};

use anyhow::Result;
use image::{ImageBuffer, Rgb};
use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Point {
    x: i64,
    y: i64,
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Map {
    width: i64,
    height: i64,
}

impl Map {
    pub fn new(width: i64, height: i64) -> Map {
        Map { width, height }
    }

    pub fn quadrants(
        &self,
    ) -> (
        HashSet<Point>,
        HashSet<Point>,
        HashSet<Point>,
        HashSet<Point>,
    ) {
        let (mut nw, mut ne, mut sw, mut se) = (
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
        );

        let half_width = self.width / 2;
        let half_height = self.height / 2;

        for y in 0..self.height {
            for x in 0..self.width {
                // Center, so skip
                if x == half_width || y == half_height {
                    continue;
                }
                let bottom = y > half_height;
                let right = x > half_width;

                let point = Point::new(x, y);
                // Add to the appropriate quadrant
                match (bottom, right) {
                    (true, true) => se.insert(point),
                    (true, false) => sw.insert(point),
                    (false, true) => ne.insert(point),
                    (false, false) => nw.insert(point),
                };
            }
        }

        (nw, ne, sw, se)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Robot {
    pos: Point,
    vel: Point,
}

impl Robot {
    pub fn new(pos: Point, velocity: Point) -> Robot {
        Robot { pos, vel: velocity }
    }
    // Advances the position of the robot by the given amount of seconds
    pub fn advance_seconds(&mut self, map: &Map, seconds: i64) -> Point {
        let adjusted_vel = Point::new(self.vel.x * seconds, self.vel.y * seconds);
        let Point { x, y } = self.pos + adjusted_vel;
        let wrapped_x = (x % map.width + map.width) % map.width;
        let wrapped_y = (y % map.height + map.height) % map.height;
        let new_pos = Point::new(wrapped_x, wrapped_y);
        self.pos = new_pos;
        self.pos
    }
}

fn main() -> Result<()> {
    let input = std::fs::read_to_string("input.txt")?;
    let robots = parse_robots(&input)?;
    let map = Map::new(101, 103);

    let part_1 = solve_part_1(&robots, &map);
    println!("Part 1: {}", part_1);

    // Part 2 doesn't return anything, but saves images
    solve_part_2(&robots, &map);

    Ok(())
}

// This probably does too much, but I'll leave my regrets for Part 2.
fn solve_part_1(robots: &[Robot], map: &Map) -> usize {
    // Create an owned copy of the robots
    let robots = robots.to_vec();

    const SECONDS: i64 = 100;

    let mut pos_map = HashMap::new();
    // Four index array representing the number of robots in each quadrant.
    let mut counters = [0; 4];
    let (nw, ne, sw, se) = map.quadrants();
    // Helper to quickly add all points to the `pos_map`
    let mut insert_points = |points: HashSet<Point>, index: usize| {
        for point in points {
            pos_map.insert(point, index);
        }
    };
    // Insert all points into the map
    insert_points(nw, 0);
    insert_points(ne, 1);
    insert_points(sw, 2);
    insert_points(se, 3);

    for mut robot in robots {
        robot.advance_seconds(map, SECONDS);
        // See if the robot is in a quadrant.
        if let Some(&index) = pos_map.get(&robot.pos) {
            // If so, increment the relevant counter.
            counters[index] += 1;
        }
    }

    // Return the product of all quadrant counter
    calculate_safety(counters).unwrap()
}

fn calculate_safety(quadrants: [usize; 4]) -> Option<usize> {
    // Multiply all the quadrant counters together
    // Returns None if the product overflows
    quadrants
        .iter()
        .try_fold(1usize, |acc, &x| acc.checked_mul(x))
}

// Part 2 doesn't return anything.
// It saves a lot of image files instead.
// Had to look at the subreddit for suggestions on how to solve this.
// Still my own implementation, but the idea is from there.
fn solve_part_2(robots: &[Robot], map: &Map) {
    // Create an owned copy of the robots
    let mut robots = robots.to_vec();

    let mut pos_map = HashMap::new();
    // Four index array representing the number of robots in each quadrant.
    let mut counters = [0; 4];
    let (nw, ne, sw, se) = map.quadrants();
    // Helper to quickly add all points to the `pos_map`
    let mut insert_points = |points: HashSet<Point>, index: usize| {
        for point in points {
            pos_map.insert(point, index);
        }
    };
    // Insert all points into the map
    insert_points(nw, 0);
    insert_points(ne, 1);
    insert_points(sw, 2);
    insert_points(se, 3);

    let mut lowest_safety = usize::MAX;

    for i in 0..100000 {
        for robot in robots.iter_mut() {
            // Advance the robot by one second
            robot.advance_seconds(map, 1);
            // See if the robot is in a quadrant.
            if let Some(&index) = pos_map.get(&robot.pos) {
                // If so, increment the relevant counter.
                counters[index] += 1;
            }
        }

        // Calculate the safety
        let Some(safety) = calculate_safety(counters) else {
            continue;
        };

        // This is an optimization, since lower safety means a denser cluster of robots.
        // So it is more likely to be the easter egg. Only saved approx 10 images this way.
        if safety < lowest_safety {
            lowest_safety = safety;
            // Save the image
            save_image(&robots, map, i);
        }

        // Reset the counters
        counters = [0; 4];
    }
}

// Saves an image of the current state of the robots to a bitmap file.
fn save_image(robots: &[Robot], map: &Map, i: usize) {
    let folder_path = "images";
    std::fs::create_dir_all(folder_path).unwrap();

    let file_name = format!("{}/image_{:05}.bmp", folder_path, i + 1);

    let mut img = ImageBuffer::new(map.width as u32, map.height as u32);

    // Fill the image with a background color (e.g., white)
    for pixel in img.pixels_mut() {
        *pixel = Rgb::<u8>([255, 255, 255]);
    }

    for robot in robots {
        let x = robot.pos.x as u32;
        let y = robot.pos.y as u32;
        img.put_pixel(x, y, Rgb([0, 0, 0])); // Assuming robots are black
    }

    println!("Saved image: {}", file_name);

    img.save(file_name).unwrap();
}

fn parse_robots(input: &str) -> Result<Vec<Robot>> {
    let re = Regex::new(r"p=(\d+),(\d+) v=(-?\d+),(-?\d+)")?;
    let mut robots = Vec::new();

    for line in input.lines() {
        if let Some(caps) = re.captures(line.trim()) {
            let px = caps[1].parse::<i64>()?;
            let py = caps[2].parse::<i64>()?;
            let vx = caps[3].parse::<i64>()?;
            let vy = caps[4].parse::<i64>()?;
            let pos = Point::new(px, py);
            let vel = Point::new(vx, vy);
            robots.push(Robot::new(pos, vel));
        }
    }

    Ok(robots)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"
        p=0,4 v=3,-3
        p=6,3 v=-1,-3
        p=10,3 v=-1,2
        p=2,0 v=2,-1
        p=0,0 v=1,3
        p=3,0 v=-2,-2
        p=7,6 v=-1,-3
        p=3,0 v=-1,-2
        p=9,3 v=2,3
        p=7,3 v=-1,2
        p=2,4 v=2,-3
        p=9,5 v=-3,-3    
    "#;

    fn create_map() -> Map {
        Map::new(11, 7)
    }

    fn create_robots() -> Vec<Robot> {
        fn robot(px: i64, py: i64, vx: i64, vy: i64) -> Robot {
            let pos = Point::new(px, py);
            let velocity = Point::new(vx, vy);

            Robot::new(pos, velocity)
        }
        vec![
            robot(0, 4, 3, -3),
            robot(6, 3, -1, -3),
            robot(10, 3, -1, 2),
            robot(2, 0, 2, -1),
            robot(0, 0, 1, 3),
            robot(3, 0, -2, -2),
            robot(7, 6, -1, -3),
            robot(3, 0, -1, -2),
            robot(9, 3, 2, 3),
            robot(7, 3, -1, 2),
            robot(2, 4, 2, -3),
            robot(9, 5, -3, -3),
        ]
    }

    #[test]
    fn test_advance_robot() {
        let mut robot = Robot::new(Point::new(2, 4), Point::new(2, -3));
        let map = create_map();
        assert_eq!(robot.advance_seconds(&map, 1), Point::new(4, 1));
        assert_eq!(robot.advance_seconds(&map, 1), Point::new(6, 5));
        assert_eq!(robot.advance_seconds(&map, 1), Point::new(8, 2));
        assert_eq!(robot.advance_seconds(&map, 1), Point::new(10, 6));
        assert_eq!(robot.advance_seconds(&map, 1), Point::new(1, 3));
    }

    #[test]
    fn test_advance_robot_single() {
        // Same as above but treated like a single big movement
        let mut robot = Robot::new(Point::new(2, 4), Point::new(2, -3));
        let map = create_map();
        assert_eq!(robot.advance_seconds(&map, 5), Point::new(1, 3));
    }

    #[test]
    fn test_quadrants() {
        let map = create_map();
        let (nw, ne, sw, se) = map.quadrants();

        assert_eq!(15, nw.iter().len());
        assert!(nw.contains(&Point::new(0, 0)));
        assert_eq!(15, ne.iter().len());
        assert_eq!(15, sw.iter().len());
        assert_eq!(15, se.iter().len());
    }

    #[test]
    fn test_parse_robots() {
        let robots = parse_robots(INPUT).unwrap();
        let expected = create_robots();
        assert_eq!(expected, robots);
    }

    #[test]
    fn test_part_1() {
        let map = create_map();
        let robots = parse_robots(INPUT).unwrap();
        let expected = 12;
        let actual = solve_part_1(&robots, &map);
        assert_eq!(expected, actual);
    }
}
