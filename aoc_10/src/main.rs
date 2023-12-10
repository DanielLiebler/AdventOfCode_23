use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use colored::Colorize;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

#[derive(PartialEq, Clone, Copy)]
enum Pipe {
    Start,
    Vertical,
    Horizontal,
    NECorner,
    NWCorner,
    SECorner,
    SWCorner,
    Ground,
}

#[derive(PartialEq, Clone, Copy)]
enum LoopType {
    Inside,
    Outside,
    Loop,
    Undefined,
}

#[derive(PartialEq, Clone, Copy)]
struct Tile {
    is_loop: LoopType,
    pipe: Pipe,
    direction: bool,
    loop_part: bool,
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    North,
    East,
    West,
    South,
}

#[derive(PartialEq, Clone, Copy)]
struct Node {
    x: usize,
    y: usize,
    from: Direction,
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Vec<Pipe> {
    let mut row: Vec<Pipe> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::start => {
                row.push(Pipe::Start);
            }
            Rule::vertical => {
                row.push(Pipe::Vertical);
            }
            Rule::horizontal => {
                row.push(Pipe::Horizontal);
            }
            Rule::ne_corner => {
                row.push(Pipe::NECorner);
            }
            Rule::nw_corner => {
                row.push(Pipe::NWCorner);
            }
            Rule::se_corner => {
                row.push(Pipe::SECorner);
            }
            Rule::sw_corner => {
                row.push(Pipe::SWCorner);
            }
            Rule::ground => {
                row.push(Pipe::Ground);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return row;
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<Vec<Pipe>> {
    let mut reports: Vec<Vec<Pipe>> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::line => reports.push(analyze_line(entry)),
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return reports;
}

fn is_in_grid(grid: &Vec<Vec<Pipe>>, x: usize, y: usize) -> bool {
    let x_len = grid.first().expect("did not expect no entry in grid").len();
    let y_len = grid.len();
    return x < x_len && y < y_len;
}

fn make_safe(grid: &Vec<Vec<Pipe>>, proposition: Node) -> Result<Node, &'static str> {
    println!("Found {}/{}", proposition.x, proposition.y);
    if is_in_grid(grid, proposition.x, proposition.y) {
        return Ok(proposition);
    } else {
        return Err("Proposition out of bounds");
    }
}

fn find_next(
    grid: &Vec<Vec<Pipe>>,
    from: Direction,
    idx: usize,
    idy: usize,
) -> Result<Node, &'static str> {
    match grid.get(idy) {
        Some(line) => match line.get(idx) {
            Some(item) => match item {
                Pipe::Horizontal => match from {
                    Direction::West => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx + 1,
                                y: idy,
                                from: Direction::West,
                            },
                        )
                    }
                    Direction::East => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx - 1,
                                y: idy,
                                from: Direction::East,
                            },
                        )
                    }
                    _ => return Err("Pipe does not match"),
                },
                Pipe::Vertical => match from {
                    Direction::North => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx,
                                y: idy + 1,
                                from: Direction::North,
                            },
                        )
                    }
                    Direction::South => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx,
                                y: idy - 1,
                                from: Direction::South,
                            },
                        )
                    }
                    _ => return Err("Pipe does not match"),
                },
                Pipe::NECorner => match from {
                    Direction::North => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx + 1,
                                y: idy,
                                from: Direction::West,
                            },
                        )
                    }
                    Direction::East => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx,
                                y: idy - 1,
                                from: Direction::South,
                            },
                        )
                    }
                    _ => return Err("Pipe does not match"),
                },
                Pipe::NWCorner => match from {
                    Direction::North => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx - 1,
                                y: idy,
                                from: Direction::East,
                            },
                        )
                    }
                    Direction::West => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx,
                                y: idy - 1,
                                from: Direction::South,
                            },
                        )
                    }
                    _ => return Err("Pipe does not match"),
                },
                Pipe::SECorner => match from {
                    Direction::South => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx + 1,
                                y: idy,
                                from: Direction::West,
                            },
                        )
                    }
                    Direction::East => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx,
                                y: idy + 1,
                                from: Direction::North,
                            },
                        )
                    }
                    _ => return Err("Pipe does not match"),
                },
                Pipe::SWCorner => match from {
                    Direction::South => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx - 1,
                                y: idy,
                                from: Direction::East,
                            },
                        )
                    }
                    Direction::West => {
                        return make_safe(
                            grid,
                            Node {
                                x: idx,
                                y: idy + 1,
                                from: Direction::North,
                            },
                        )
                    }
                    _ => return Err("Pipe does not match"),
                },
                Pipe::Ground => return Err("Did not expect Ground"),
                Pipe::Start => return Err("Did not expect Start"),
            },
            None => return Err("Could not find item"),
        },
        None => return Err("Could not find line"),
    }
}

fn is_neighbor(grid: &Vec<Vec<Pipe>>, x: usize, y: usize, dir: Direction) -> bool {
    let mut x = x;
    let mut y = y;
    let mut from = dir;
    match dir {
        Direction::East => {
            x += 1;
            from = Direction::West;
        }
        Direction::West => {
            if x == 0 {
                return false;
            }
            x -= 1;
            from = Direction::East;
        }
        Direction::North => {
            if y == 0 {
                return false;
            }
            y -= 1;
            from = Direction::South;
        }
        Direction::South => {
            y += 1;
            from = Direction::North;
        }
    }
    if is_in_grid(grid, x, y) {
        match grid.get(y) {
            Some(line) => match line.get(x) {
                Some(entry) => match entry {
                    Pipe::Ground => return false,
                    Pipe::Start => return false,
                    Pipe::Vertical => return from == Direction::North || from == Direction::South,
                    Pipe::Horizontal => return from == Direction::West || from == Direction::East,
                    Pipe::NECorner => return from == Direction::North || from == Direction::East,
                    Pipe::NWCorner => return from == Direction::North || from == Direction::West,
                    Pipe::SECorner => return from == Direction::South || from == Direction::East,
                    Pipe::SWCorner => return from == Direction::South || from == Direction::West,
                },
                None => return false,
            },
            None => return false,
        }
    } else {
        return false;
    }
}
fn find_first_neighbors(
    grid: &Vec<Vec<Pipe>>,
) -> Result<(Vec<Node>, (usize, usize)), &'static str> {
    let start: Option<(usize, usize)> = grid
        .iter()
        .enumerate()
        .map(|(i, line)| {
            (
                i,
                line.iter()
                    .enumerate()
                    .find(|(_j, item)| **item == Pipe::Start)
                    .map(|(j, _item)| j),
            )
        })
        .filter_map(|(i, j)| match j {
            Some(j) => Some((i, j)),
            None => None,
        })
        .next();

    match start {
        Some((j, i)) => {
            let mut result: Vec<Node> = Vec::new();
            println!("Start at {i}/{j}");

            if is_neighbor(grid, i, j, Direction::North) {
                result.push(Node {
                    x: i,
                    y: j - 1,
                    from: Direction::South,
                });
            }
            if is_neighbor(grid, i, j, Direction::East) {
                result.push(Node {
                    x: i + 1,
                    y: j,
                    from: Direction::West,
                });
            }
            if is_neighbor(grid, i, j, Direction::South) {
                result.push(Node {
                    x: i,
                    y: j + 1,
                    from: Direction::North,
                });
            }
            if is_neighbor(grid, i, j, Direction::West) {
                result.push(Node {
                    x: i - 1,
                    y: j,
                    from: Direction::East,
                });
            }

            if result.len() == 2 {
                return Ok((result, (i, j)));
            } else {
                println!("Found {} nbs", result.len());
                for n in result {
                    println!("NB: {}/{}", n.x, n.y);
                }
                return Err("Found not exactly two valid start neighbors");
            }
        }
        None => return Err("Did not find Start"),
    }
}

fn nodes_match(a: &Node, b: &Node) -> bool {
    a.x == b.x && a.y == b.y
}

fn add_to_tile_grid(
    tile_grid: &mut Vec<Vec<Tile>>,
    next_node: &Node,
    grid: &Vec<Vec<Pipe>>,
    direction: bool,
) {
    let tile = tile_grid
        .get_mut(next_node.y)
        .expect("tile_grid not big enough")
        .get_mut(next_node.x)
        .expect("tile_grid not big enough");
    tile.is_loop = LoopType::Loop;
    tile.pipe = grid
        .get(next_node.y)
        .expect("grid not big enough")
        .get(next_node.x)
        .expect("grid not big enough")
        .clone();
    tile.loop_part = direction;
    tile.direction = match (tile.pipe, next_node.from) {
        (Pipe::Vertical, Direction::North) => direction,
        (Pipe::Vertical, Direction::South) => !direction,
        (Pipe::Vertical, _) => {
            println!("Did not expect E/W for V");
            true
        }
        (Pipe::Horizontal, Direction::East) => !direction,
        (Pipe::Horizontal, Direction::West) => direction,
        (Pipe::Horizontal, _) => {
            println!("Did not expect N/S for H");
            true
        }
        (Pipe::NECorner, Direction::North) => direction,
        (Pipe::NECorner, Direction::East) => !direction,
        (Pipe::NECorner, _) => {
            println!("Did not expect S/W for NE");
            true
        }
        (Pipe::NWCorner, Direction::North) => direction,
        (Pipe::NWCorner, Direction::West) => !direction,
        (Pipe::NWCorner, _) => {
            println!("Did not expect S/E for NW");
            true
        }
        (Pipe::SECorner, Direction::East) => !direction,
        (Pipe::SECorner, Direction::South) => direction,
        (Pipe::SECorner, _) => {
            println!("Did not expect N/W for SE");
            true
        }
        (Pipe::SWCorner, Direction::West) => direction,
        (Pipe::SWCorner, Direction::South) => !direction,
        (Pipe::SWCorner, _) => {
            println!("Did not expect N/E for SW");
            true
        }
        (Pipe::Ground, _) => {
            println!("Did not expect ground");
            true
        }
        (Pipe::Start, _) => {
            println!("Did not expect start");
            true
        }
    };
}

fn swap_in_outside(val: &mut LoopType) {
    if *val == LoopType::Outside {
        *val = LoopType::Inside;
    } else if *val == LoopType::Inside {
        *val = LoopType::Outside;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let grid = analyze_file(&mut result);
            let x_len = grid.first().expect("did not expect no entry in grid").len();
            let y_len = grid.len();

            match find_first_neighbors(&grid) {
                Ok((first_neighbors, start)) => {
                    for n in &first_neighbors {
                        println!("First: {}/{}", n.x, n.y);
                    }
                    let mut f_n_iter = first_neighbors.iter();
                    let mut a = f_n_iter.next().expect("could not find neighbor a").clone();
                    let mut b = f_n_iter.next().expect("could not find neighbor b").clone();
                    let mut steps = 1;

                    let mut tile_grid: Vec<Vec<Tile>> = vec![
                        vec![
                            Tile {
                                is_loop: LoopType::Undefined,
                                pipe: Pipe::Ground,
                                direction: false,
                                loop_part: false,
                            };
                            x_len
                        ];
                        y_len
                    ];

                    let start_tile = tile_grid
                        .get_mut(start.1)
                        .expect("could not access start in tile_grid")
                        .get_mut(start.0)
                        .expect("could not access start in tile_grid");
                    start_tile.is_loop = LoopType::Loop;
                    start_tile.pipe = Pipe::Start;
                    start_tile.direction = true;
                    start_tile.loop_part = true;

                    add_to_tile_grid(&mut tile_grid, &a, &grid, true);
                    add_to_tile_grid(&mut tile_grid, &b, &grid, false);

                    loop {
                        let next_a = match find_next(&grid, a.from, a.x, a.y) {
                            Ok(next_a) => next_a,
                            Err(_e) => {
                                println!("Error finding next: {_e}");
                                return;
                            }
                        };
                        let next_b = match find_next(&grid, b.from, b.x, b.y) {
                            Ok(next_b) => next_b,
                            Err(_e) => {
                                println!("Error finding next: {_e}");
                                return;
                            }
                        };

                        steps += 1;
                        if nodes_match(&a, &next_b) {
                            println!("{steps} steps(b)");
                            break;
                        } else if nodes_match(&next_a, &b) {
                            println!("{steps} steps(a)");
                            break;
                        } else if nodes_match(&next_a, &next_b) {
                            println!("{steps} steps(both)");
                            add_to_tile_grid(&mut tile_grid, &next_a, &grid, true);
                            break;
                        }

                        add_to_tile_grid(&mut tile_grid, &next_a, &grid, true);
                        add_to_tile_grid(&mut tile_grid, &next_b, &grid, false);
                        a = next_a;
                        b = next_b;
                    }

                    // Calculate inside and outside definition
                    for line in tile_grid.iter_mut() {
                        let mut state = LoopType::Outside;
                        let mut creep_upper_half = true;
                        for tile in line.iter_mut() {
                            match tile.is_loop {
                                LoopType::Loop => match (tile.pipe, tile.direction) {
                                    (Pipe::Start, _) => {
                                        println!("TODO");
                                        creep_upper_half = false;
                                        //state = LoopType::Undefined;
                                    }
                                    (Pipe::Vertical, _) => {
                                        swap_in_outside(&mut state);
                                    }
                                    (Pipe::Horizontal, _) => {}
                                    (Pipe::NECorner, _) => {
                                        creep_upper_half = false;
                                    }
                                    (Pipe::NWCorner, _) => {
                                        if creep_upper_half {
                                            swap_in_outside(&mut state);
                                        }
                                    }
                                    (Pipe::SECorner, _) => {
                                        creep_upper_half = true;
                                    }
                                    (Pipe::SWCorner, _) => {
                                        if !creep_upper_half {
                                            swap_in_outside(&mut state);
                                        }
                                    }
                                    (Pipe::Ground, _) => println!("Should not find ground on loop"),
                                },
                                LoopType::Undefined => tile.is_loop = state.clone(),
                                LoopType::Inside => println!("Should not be set yet"),
                                LoopType::Outside => println!("Should not be set yet"),
                            }
                        }
                    }

                    let count = tile_grid.iter().fold((0, 0), |accu: (usize, usize), line| {
                        let res = line.iter().fold((0, 0), |accu: (usize, usize), t| {
                            if t.is_loop == LoopType::Inside {
                                (accu.0 + 1, accu.1)
                            } else if t.is_loop == LoopType::Outside {
                                (accu.0, accu.1 + 1)
                            } else {
                                (accu.0, accu.1)
                            }
                        });
                        (accu.0 + res.0, accu.1 + res.1)
                    });
                    println!("Found {} Inside, {} Outside", count.0, count.1);

                    // Visualize
                    for line in &tile_grid {
                        for tile in line {
                            if tile.is_loop == LoopType::Loop {
                                let (p, col) = match (tile.pipe, tile.loop_part) {
                                    (Pipe::Start, d) => ("S", d),
                                    (Pipe::Vertical, d) => ("|", d),
                                    (Pipe::Horizontal, d) => ("-", d),
                                    (Pipe::NECorner, d) => ("L", d),
                                    (Pipe::NWCorner, d) => ("J", d),
                                    (Pipe::SECorner, d) => ("F", d),
                                    (Pipe::SWCorner, d) => ("7", d),
                                    (Pipe::Ground, d) => ("X", d),
                                };
                                if p == "S" {
                                    print!("{}", "S".green());
                                } else if col {
                                    print!("{}", p.blue());
                                } else {
                                    print!("{}", p.red());
                                }
                            } else {
                                match tile.is_loop {
                                    LoopType::Inside => print!("{}", ".".yellow()),
                                    LoopType::Outside => print!("{}", ".".cyan()),
                                    LoopType::Loop => print!("{}", "X".green()),
                                    LoopType::Undefined => print!("{}", "X".green()),
                                }
                            }
                        }
                        println!("");
                    }

                    // TODO: Count
                }
                Err(_e) => println!("Err {_e}"),
            }
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
