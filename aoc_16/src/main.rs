use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

#[derive(PartialEq, Clone, Eq)]
enum TileContent {
    Empty,
    SplitterHorizontal,
    SplitterVertical,
    MirrorTopLeft,
    MirrorTopRight,
}

#[derive(PartialEq, Clone, Eq)]
struct Tile {
    content: TileContent,
    power: usize,
    walked_up: bool,
    walked_left: bool,
    walked_right: bool,
    walked_down: bool,
}

#[derive(PartialEq, Clone, Eq)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}
#[derive(PartialEq, Clone, Eq)]
struct Ray {
    x: usize,
    y: usize,
    direction: Direction,
}
#[derive(PartialEq, Clone, Eq)]
struct Dimension {
    x: usize,
    y: usize,
}

fn analyze_tile(parsed: Pair<'_, Rule>) -> Result<Tile, &'static str> {
    let mut tile: Option<Tile> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::empty => {
                tile = Some(Tile {
                    content: TileContent::Empty,
                    power: 0,
                    walked_up: false,
                    walked_left: false,
                    walked_right: false,
                    walked_down: false,
                })
            }
            Rule::splitter_h => {
                tile = Some(Tile {
                    content: TileContent::SplitterHorizontal,
                    power: 0,
                    walked_up: false,
                    walked_left: false,
                    walked_right: false,
                    walked_down: false,
                })
            }
            Rule::splitter_v => {
                tile = Some(Tile {
                    content: TileContent::SplitterVertical,
                    power: 0,
                    walked_up: false,
                    walked_left: false,
                    walked_right: false,
                    walked_down: false,
                })
            }
            Rule::mirror_tl => {
                tile = Some(Tile {
                    content: TileContent::MirrorTopLeft,
                    power: 0,
                    walked_up: false,
                    walked_left: false,
                    walked_right: false,
                    walked_down: false,
                })
            }
            Rule::mirror_tr => {
                tile = Some(Tile {
                    content: TileContent::MirrorTopRight,
                    power: 0,
                    walked_up: false,
                    walked_left: false,
                    walked_right: false,
                    walked_down: false,
                })
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(entry) {}", entry.as_str());
            }
        }
    }
    match tile {
        Some(tile) => Ok(tile),
        None => Err("Did not find Tile specification"),
    }
}
fn analyze_line(parsed: Pair<'_, Rule>) -> Vec<Tile> {
    let mut line: Vec<Tile> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::tile => match analyze_tile(entry) {
                Ok(tile) => line.push(tile),
                Err(error) => println!("Error parsing tile: '{error}'"),
            },
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(entry) {}", entry.as_str());
            }
        }
    }
    return line;
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<Vec<Tile>> {
    let mut grid: Vec<Vec<Tile>> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::line => {
                grid.push(analyze_line(entry));
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return grid;
}

fn get_tile(grid: &mut Vec<Vec<Tile>>, x: usize, y: usize) -> &mut Tile {
    return grid
        .get_mut(y)
        .and_then(|f| f.get_mut(x))
        .expect("Could not find Tile");
}
fn get_next_coords(ray: &Ray, dimensions: &Dimension) -> Option<(usize, usize)> {
    match ray.direction {
        Direction::Up => match ray.y {
            0 => return None,
            _ => return Some((ray.x, ray.y - 1)),
        },
        Direction::Left => match ray.x {
            0 => return None,
            _ => return Some((ray.x - 1, ray.y)),
        },
        Direction::Down => match ray.y {
            y if y < dimensions.y - 1 => return Some((ray.x, ray.y + 1)),
            _ => return None,
        },
        Direction::Right => match ray.x {
            x if x < dimensions.x - 1 => return Some((ray.x + 1, ray.y)),
            _ => return None,
        },
    }
}
fn move_ray(
    ray: &mut Ray,
    dimensions: &Dimension,
    remove_ray: &mut bool,
    grid: &mut Vec<Vec<Tile>>,
) {
    let direction = ray.direction.clone();
    match get_next_coords(ray, dimensions) {
        Some((x, y)) => {
            let tile = get_tile(grid, x, y);
            let walked_marker = match direction {
                Direction::Up => &mut tile.walked_up,
                Direction::Left => &mut tile.walked_left,
                Direction::Down => &mut tile.walked_down,
                Direction::Right => &mut tile.walked_right,
            };
            if *walked_marker {
                // already walked ray there
                *remove_ray = true;
            } else {
                ray.x = x;
                ray.y = y;
                *walked_marker = true;
            }
        }
        None => *remove_ray = true,
    };
}
fn calculate_powers(
    grid: &mut Vec<Vec<Tile>>,
    dimensions: &Dimension,
    start_x: usize,
    start_y: usize,
    start_direction: Direction,
) {
    let mut rays: Vec<Ray> = vec![Ray {
        x: start_x,
        y: start_y,
        direction: start_direction,
    }];

    loop {
        let mut remove_ray = false;
        let mut add_ray: Option<Ray> = None;
        match rays.first_mut() {
            Some(ray) => {
                let tile = get_tile(grid, ray.x, ray.y);
                tile.power += 1;
                match tile.content {
                    TileContent::Empty => move_ray(ray, &dimensions, &mut remove_ray, grid),
                    TileContent::SplitterHorizontal => match ray.direction {
                        Direction::Up | Direction::Down => {
                            let mut ray2 = ray.clone();
                            ray.direction = Direction::Left;
                            ray2.direction = Direction::Right;
                            let mut remove_ray_1 = false;
                            let mut remove_ray_2 = false;

                            move_ray(ray, &dimensions, &mut remove_ray_1, grid);
                            move_ray(&mut ray2, &dimensions, &mut remove_ray_2, grid);

                            match (remove_ray_1, remove_ray_2) {
                                (true, true) => remove_ray = true,
                                (true, false) => {
                                    ray.direction = ray2.direction;
                                    ray.x = ray2.x;
                                    ray.y = ray2.y;
                                }
                                (false, true) => {}
                                (false, false) => add_ray = Some(ray2),
                            }
                        }
                        Direction::Left | Direction::Right => {
                            move_ray(ray, &dimensions, &mut remove_ray, grid)
                        }
                    },
                    TileContent::SplitterVertical => match ray.direction {
                        Direction::Left | Direction::Right => {
                            let mut ray2 = ray.clone();
                            ray.direction = Direction::Up;
                            ray2.direction = Direction::Down;
                            let mut remove_ray_1 = false;
                            let mut remove_ray_2 = false;

                            move_ray(ray, &dimensions, &mut remove_ray_1, grid);
                            move_ray(&mut ray2, &dimensions, &mut remove_ray_2, grid);

                            match (remove_ray_1, remove_ray_2) {
                                (true, true) => remove_ray = true,
                                (true, false) => {
                                    ray.direction = ray2.direction;
                                    ray.x = ray2.x;
                                    ray.y = ray2.y;
                                }
                                (false, true) => {}
                                (false, false) => add_ray = Some(ray2),
                            }
                        }
                        Direction::Up | Direction::Down => {
                            move_ray(ray, &dimensions, &mut remove_ray, grid)
                        }
                    },
                    TileContent::MirrorTopLeft => match ray.direction {
                        Direction::Up => {
                            ray.direction = Direction::Right;
                            move_ray(ray, &dimensions, &mut remove_ray, grid);
                        }
                        Direction::Left => {
                            ray.direction = Direction::Down;
                            move_ray(ray, &dimensions, &mut remove_ray, grid);
                        }
                        Direction::Down => {
                            ray.direction = Direction::Left;
                            move_ray(ray, &dimensions, &mut remove_ray, grid);
                        }
                        Direction::Right => {
                            ray.direction = Direction::Up;
                            move_ray(ray, &dimensions, &mut remove_ray, grid);
                        }
                    },
                    TileContent::MirrorTopRight => match ray.direction {
                        Direction::Up => {
                            ray.direction = Direction::Left;
                            move_ray(ray, &dimensions, &mut remove_ray, grid);
                        }
                        Direction::Left => {
                            ray.direction = Direction::Up;
                            move_ray(ray, &dimensions, &mut remove_ray, grid);
                        }
                        Direction::Down => {
                            ray.direction = Direction::Right;
                            move_ray(ray, &dimensions, &mut remove_ray, grid);
                        }
                        Direction::Right => {
                            ray.direction = Direction::Down;
                            move_ray(ray, &dimensions, &mut remove_ray, grid);
                        }
                    },
                }
            }
            None => break,
        }
        if remove_ray {
            rays.remove(0);
        }
        match add_ray {
            Some(ray) => rays.push(ray),
            None => {}
        }
    }
}

fn sum_power(grid: &Vec<Vec<Tile>>) -> usize {
    return grid
        .iter()
        .map(|line| line.iter().filter(|tile| tile.power >= 1).count())
        .sum();
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let mut grid = analyze_file(&mut result);

            let dimensions = Dimension {
                x: grid
                    .first()
                    .expect("could not access first line of grid")
                    .len(),
                y: grid.len(),
            };
            let mut max_power = 0;
            for x in 0..dimensions.x {
                let mut power_down_grid = grid.to_vec();
                calculate_powers(&mut power_down_grid, &dimensions, x, 0, Direction::Down);
                let mut power_up_grid = grid.to_vec();
                calculate_powers(
                    &mut power_up_grid,
                    &dimensions,
                    x,
                    dimensions.x - 1,
                    Direction::Up,
                );
                let power_down = sum_power(&power_down_grid);
                let power_up = sum_power(&power_up_grid);
                if power_down > max_power {
                    max_power = power_down;
                }
                if power_up > max_power {
                    max_power = power_up;
                }
            }
            for y in 0..dimensions.y {
                let mut power_right_grid = grid.to_vec();
                calculate_powers(&mut power_right_grid, &dimensions, 0, y, Direction::Right);
                let mut power_left_grid = grid.to_vec();
                calculate_powers(
                    &mut power_left_grid,
                    &dimensions,
                    dimensions.y - 1,
                    y,
                    Direction::Left,
                );
                let power_right = sum_power(&power_right_grid);
                let power_left = sum_power(&power_left_grid);
                if power_right > max_power {
                    max_power = power_right;
                }
                if power_left > max_power {
                    max_power = power_left;
                }
            }

            println!("Max power sum is {}", max_power);
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
