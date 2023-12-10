use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

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
fn find_first_neighbors(grid: &Vec<Vec<Pipe>>) -> Result<Vec<Node>, &'static str> {
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
                return Ok(result);
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
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let grid = analyze_file(&mut result);

            match find_first_neighbors(&grid) {
                Ok(first_neighbors) => {
                    for n in &first_neighbors {
                        println!("First: {}/{}", n.x, n.y);
                    }
                    let mut f_n_iter = first_neighbors.iter();
                    let mut a = f_n_iter.next().expect("could not find neighbor a").clone();
                    let mut b = f_n_iter.next().expect("could not find neighbor b").clone();
                    let mut steps = 1;
                    loop {
                        let next_a = match find_next(&grid, a.from, a.x, a.y) {
                            Ok(next_a) => next_a,
                            Err(_e) => {
                                println!("Error finding next: {_e}");
                                break;
                            }
                        };
                        let next_b = match find_next(&grid, b.from, b.x, b.y) {
                            Ok(next_b) => next_b,
                            Err(_e) => {
                                println!("Error finding next: {_e}");
                                break;
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
                            break;
                        }
                        a = next_a;
                        b = next_b;
                    }
                }
                Err(_e) => println!("Err {_e}"),
            }
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
