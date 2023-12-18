use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

#[derive(PartialEq, Clone, Eq, Copy)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

struct DigPlanEntry {
    direction: Direction,
    length: i64,
}

fn analyze_direction_hex(parsed: Pair<'_, Rule>) -> Option<Direction> {
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::up_hex => {
                return Some(Direction::Up);
            }
            Rule::down_hex => {
                return Some(Direction::Down);
            }
            Rule::left_hex => {
                return Some(Direction::Left);
            }
            Rule::right_hex => {
                return Some(Direction::Right);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(entry) {}", entry.as_str());
            }
        }
    }
    return None;
}
fn analyze_direction(parsed: Pair<'_, Rule>) -> Option<Direction> {
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::up => {
                return Some(Direction::Up);
            }
            Rule::down => {
                return Some(Direction::Down);
            }
            Rule::left => {
                return Some(Direction::Left);
            }
            Rule::right => {
                return Some(Direction::Right);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(entry) {}", entry.as_str());
            }
        }
    }
    return None;
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Result<DigPlanEntry, &'static str> {
    let mut direction: Option<Direction> = None;
    let mut count: Option<i64> = None;
    let mut direction_hex: Option<Direction> = None;
    let mut count_hex: Option<i64> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::direction => {
                direction = analyze_direction(entry);
            }
            Rule::number => {
                count = Some(
                    entry
                        .as_str()
                        .parse()
                        .expect("could not parse sequence entry"),
                );
            }
            Rule::hex => {
                count_hex = Some(
                    i64::from_str_radix(entry.as_str(), 16)
                        .expect("could not parse sequence entry"),
                );
            }
            Rule::hex_dir => {
                direction_hex = analyze_direction_hex(entry);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(entry) {}", entry.as_str());
            }
        }
    }

    //match (direction, count) {
    match (direction_hex, count_hex) {
        (Some(d), Some(c)) => {
            return Ok(DigPlanEntry {
                direction: d,
                length: c,
            })
        }
        (_, _) => return Err("Error parsing line"),
    }
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<DigPlanEntry> {
    let mut dig_plan: Vec<DigPlanEntry> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::line => match analyze_line(entry) {
                Ok(entry) => dig_plan.push(entry),
                Err(_e) => println!("Error: {}", _e),
            },
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return dig_plan;
}

fn get_turn_direction(dir1: &Direction, dir2: &Direction) -> i32 {
    match (dir1, dir2) {
        (Direction::Up, Direction::Up) => println!("No Direction Change (UU)"),
        (Direction::Up, Direction::Left) => return -1,
        (Direction::Up, Direction::Down) => println!("Wrong Direction Change (UD)"),
        (Direction::Up, Direction::Right) => return 1,
        (Direction::Left, Direction::Up) => return 1,
        (Direction::Left, Direction::Left) => println!("No Direction Change (LL)"),
        (Direction::Left, Direction::Down) => return -1,
        (Direction::Left, Direction::Right) => println!("Wrong Direction Change (LR)"),
        (Direction::Down, Direction::Up) => println!("Wrong Direction Change (DU)"),
        (Direction::Down, Direction::Left) => return 1,
        (Direction::Down, Direction::Down) => println!("No Direction Change (DD)"),
        (Direction::Down, Direction::Right) => return -1,
        (Direction::Right, Direction::Up) => return -1,
        (Direction::Right, Direction::Left) => println!("Wrong Direction Change (RL)"),
        (Direction::Right, Direction::Down) => return 1,
        (Direction::Right, Direction::Right) => println!("No Direction Change (RR)"),
    };
    return 0;
}
fn get_loop_direction(dig_plan: &Vec<DigPlanEntry>) -> bool {
    // true = cw, false = ccw
    let mut previous_direction: Option<Direction> = None;
    let mut first_direction = Direction::Up;
    let mut right_turns = 0;
    for entry in dig_plan {
        match previous_direction {
            Some(dir) => {
                right_turns += get_turn_direction(&dir, &entry.direction);
                previous_direction = Some(entry.direction);
            }
            None => {
                first_direction = entry.direction;
                previous_direction = Some(entry.direction);
            }
        }
    }
    match previous_direction {
        Some(previous_direction) => {
            let last_addition = get_turn_direction(&previous_direction, &first_direction);
            match right_turns + last_addition {
                4 => return true,
                -4 => return false,
                i => println!("Loop does not close: {i}"),
            }
        }
        None => return false,
    }

    return false;
}

fn as_aligned_direction(anchor: &Direction, value: &Direction) -> Direction {
    match (anchor, value) {
        (Direction::Up, Direction::Up) => return Direction::Up,
        (Direction::Up, Direction::Left) => return Direction::Left,
        (Direction::Up, Direction::Down) => return Direction::Down,
        (Direction::Up, Direction::Right) => return Direction::Right,
        (Direction::Left, Direction::Up) => return Direction::Right,
        (Direction::Left, Direction::Left) => return Direction::Up,
        (Direction::Left, Direction::Down) => return Direction::Left,
        (Direction::Left, Direction::Right) => return Direction::Down,
        (Direction::Down, Direction::Up) => return Direction::Down,
        (Direction::Down, Direction::Left) => return Direction::Right,
        (Direction::Down, Direction::Down) => return Direction::Up,
        (Direction::Down, Direction::Right) => return Direction::Left,
        (Direction::Right, Direction::Up) => return Direction::Left,
        (Direction::Right, Direction::Left) => return Direction::Down,
        (Direction::Right, Direction::Down) => return Direction::Right,
        (Direction::Right, Direction::Right) => return Direction::Up,
    }
}

fn sum_area_cw(dig_plan: &Vec<DigPlanEntry>) -> i64 {
    let mut previous_direction: Option<Direction> = None;
    let mut first_direction = Direction::Up;

    let mut area: i64 = 0;
    let mut height: i64 = 0;
    let mut offset: i64 = 0;

    for entry in dig_plan {
        match previous_direction {
            Some(dir) => {
                let aligned = as_aligned_direction(&first_direction, &entry.direction);
                print!(
                    "{}:{}",
                    match aligned {
                        Direction::Up => "U",
                        Direction::Left => "L",
                        Direction::Down => "D",
                        Direction::Right => "R",
                    },
                    entry.length
                );

                match aligned {
                    Direction::Up => {
                        height += entry.length;
                        if offset > 0 {
                            area += offset * entry.length
                        }
                    }
                    Direction::Left => {
                        offset += entry.length;
                    }
                    Direction::Down => {
                        height -= entry.length;
                        area += entry.length;
                        if offset > 0 {
                            area -= offset * entry.length;
                        }
                        /*if height < 0 {
                            height = -height;
                            println!("Swapping anchor");
                            first_direction = match first_direction {
                                Direction::Up => Direction::Down,
                                Direction::Left => Direction::Right,
                                Direction::Down => Direction::Up,
                                Direction::Right => Direction::Left,
                            }
                        }*/
                    }
                    Direction::Right => {
                        offset -= entry.length;
                        area += entry.length;
                        if offset < 0 {
                            area += (height - 1) * (-offset);
                            offset = 0;
                        }
                    }
                }

                previous_direction = Some(entry.direction);
                println!(" - H{} O{} A{}", height, offset, area);
            }
            None => {
                first_direction = entry.direction;
                height = entry.length + 1;
                previous_direction = Some(entry.direction);
            }
        }
    }

    return area + 1;
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let mut dig_plan = analyze_file(&mut result);

            let loop_dir = get_loop_direction(&dig_plan);

            let result = sum_area_cw(&dig_plan);
            println!("result is {} (Loop CW:{})", result, loop_dir);
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
