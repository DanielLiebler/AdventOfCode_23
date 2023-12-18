use std::cmp::min;
use std::cmp::Ordering;
use std::collections::HashMap;
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
struct Tile {
    heat_loss: usize,
}
#[derive(PartialEq, Clone, Eq)]
struct Dimension {
    x: usize,
    y: usize,
}

#[derive(PartialEq, Clone, Eq)]
struct PathHead {
    current_direction: Direction,
    straight_len: usize,
    heat_loss: usize,
    x: usize,
    y: usize,
    len: usize,
    path: Vec<(usize, usize, Direction, usize)>,
}

#[derive(PartialEq, Clone, Eq)]
struct DijkstraFrom {
    x: usize,
    y: usize,
    direction: Direction,
    straight_len: usize,
    heat_loss: usize,
}

#[derive(PartialEq, Clone, Eq)]
struct DijkstraNode {
    heat_loss: usize,
    x: usize,
    y: usize,
    prev_heat_loss: usize,
    straight_len: usize,
    direction: Direction,
}

#[derive(PartialEq, Clone, Eq, Copy)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Vec<Tile> {
    let mut line: Vec<Tile> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::digit => {
                line.push(Tile {
                    heat_loss: entry
                        .as_str()
                        .parse()
                        .expect("could not parse sequence entry"),
                });
            }
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

fn new_path_head(
    grid: &mut Vec<Vec<Tile>>,
    head: &PathHead,
    direction: Direction,
    dimensions: &Dimension,
) -> Option<PathHead> {
    match direction {
        Direction::Up => {
            if head.y == 0 {
                //println!("Up out of bounds");
                return None;
            }
            return if direction != head.current_direction || head.straight_len + 1 <= 3 {
                let new_straight_len = if direction == head.current_direction {
                    head.straight_len + 1
                } else {
                    1
                };
                Some(PathHead {
                    current_direction: direction.clone(),
                    straight_len: new_straight_len,
                    heat_loss: head.heat_loss + get_tile(grid, head.x, head.y - 1).heat_loss,
                    x: head.x,
                    y: head.y - 1,
                    len: head.len + 1,
                    path: head.path.to_owned(),
                })
            } else {
                //println!("Too long straight for Up");
                None
            };
        }
        Direction::Left => {
            if head.x == 0 {
                //println!("Left out of bounds");
                return None;
            }
            return if direction != head.current_direction || head.straight_len + 1 <= 3 {
                let new_straight_len = if direction == head.current_direction {
                    head.straight_len + 1
                } else {
                    1
                };
                Some(PathHead {
                    current_direction: direction.clone(),
                    straight_len: new_straight_len,
                    heat_loss: head.heat_loss + get_tile(grid, head.x - 1, head.y).heat_loss,
                    x: head.x - 1,
                    y: head.y,
                    len: head.len + 1,
                    path: head.path.to_owned(),
                })
            } else {
                //println!("Too long straight for Left");
                None
            };
        }
        Direction::Down => {
            if head.y >= dimensions.y - 1 {
                //println!("Down out of bounds");
                return None;
            }
            return if direction != head.current_direction || head.straight_len + 1 <= 3 {
                let new_straight_len = if direction == head.current_direction {
                    head.straight_len + 1
                } else {
                    1
                };
                Some(PathHead {
                    current_direction: direction.clone(),
                    straight_len: new_straight_len,
                    heat_loss: head.heat_loss + get_tile(grid, head.x, head.y + 1).heat_loss,
                    x: head.x,
                    y: head.y + 1,
                    len: head.len + 1,
                    path: head.path.to_owned(),
                })
            } else {
                //println!("Too long straight for Down");
                None
            };
        }
        Direction::Right => {
            if head.x >= dimensions.x - 1 {
                //println!("Right out of bounds");
                return None;
            }
            return if direction != head.current_direction || head.straight_len + 1 <= 3 {
                Some(PathHead {
                    current_direction: direction.clone(),
                    straight_len: if direction == head.current_direction {
                        head.straight_len + 1
                    } else {
                        1
                    },
                    heat_loss: head.heat_loss + get_tile(grid, head.x + 1, head.y).heat_loss,
                    x: head.x + 1,
                    y: head.y,
                    len: head.len + 1,
                    path: head.path.to_owned(),
                })
            } else {
                //println!("Too long straight for Right");
                None
            };
        }
    }
}

fn get_min_remaining_cost(
    grid: &Vec<Vec<Tile>>,
    remaining_cache: &mut HashMap<(usize, usize), usize>,
    x: usize,
    y: usize,
) -> usize {
    let key = (x, y);
    match remaining_cache.get(&key) {
        Some(entry) => return *entry,
        None => {
            let val = grid
                .get(y..)
                .expect("Could not slice grid")
                .iter()
                .fold(10, |accu, line| {
                    line.iter().fold(accu, |accu, tile| {
                        if tile.heat_loss < accu {
                            tile.heat_loss
                        } else {
                            accu
                        }
                    })
                });
            remaining_cache.insert(key, val);
            return val;
        }
    }
}

fn get_value(
    head: &PathHead,
    grid: &Vec<Vec<Tile>>,
    dimensions: &Dimension,
    remaining_cache: &mut HashMap<(usize, usize), usize>,
) -> usize {
    return head.heat_loss;
    let remaining_x = dimensions.x - head.x - 1;
    let remaining_y = dimensions.y - head.y - 1;

    return head.heat_loss
        + min(
            get_min_remaining_cost(grid, remaining_cache, head.x, 0) * (remaining_x + remaining_y),
            get_min_remaining_cost(grid, remaining_cache, 0, head.y) * (remaining_x + remaining_y),
        );
}

fn compare_value(
    a: &PathHead,
    b: &PathHead,
    grid: &Vec<Vec<Tile>>,
    dimensions: &Dimension,
    remaining_cache: &mut HashMap<(usize, usize), usize>,
) -> Ordering {
    let a = get_value(a, grid, dimensions, remaining_cache);
    let b = get_value(b, grid, dimensions, remaining_cache);
    return b.cmp(&a);
}

fn find_path(grid: &mut Vec<Vec<Tile>>, dimensions: &Dimension) -> usize {
    let start_a = PathHead {
        current_direction: Direction::Right,
        straight_len: 1,
        heat_loss: get_tile(grid, 1, 0).heat_loss,
        x: 1,
        y: 0,
        len: 1,
        path: vec![(1, 0, Direction::Right, 1)],
    };
    let start_b = PathHead {
        current_direction: Direction::Down,
        straight_len: 1,
        heat_loss: get_tile(grid, 0, 1).heat_loss,
        x: 0,
        y: 1,
        len: 1,
        path: vec![(0, 1, Direction::Down, 1)],
    };
    let mut results: Vec<PathHead> = Vec::new();
    let mut max_val = 0;
    let mut min_path_left = 100000000;
    let mut remaining_cache: HashMap<(usize, usize), usize> = HashMap::new();

    let mut explorations: HashMap<usize, Vec<PathHead>> = HashMap::new();
    explorations.insert(
        get_value(&start_a, grid, dimensions, &mut remaining_cache),
        vec![start_a],
    );
    explorations.insert(
        get_value(&start_b, grid, dimensions, &mut remaining_cache),
        vec![start_b],
    );
    let mut explored: Vec<(usize, usize)> = Vec::new();

    let mut last_key = 0;
    loop {
        let key = last_key;
        //let same = explorations.keys().filter(|k| **k == last_key).count();
        //let lower = explorations.keys().filter(|k| **k < last_key).count();
        //let higher = explorations.keys().filter(|k| **k > last_key).count();
        //println!("Got Key {} ({}|{}|{})", key, lower, same, higher);

        match explorations.get_mut(&key) {
            Some(list) => {
                let remaining_items = list.len();
                let first = list.pop().expect("Could not find first item");
                if remaining_items <= 1 {
                    explorations.remove(&key);
                }
                if explored
                    .iter()
                    .find(|(x, y)| *x == first.x && *y == first.y)
                    .is_some()
                {
                    continue;
                }
                explored.push((first.x, first.y));
                println!("{}/{}: {}", first.x, first.y, first.heat_loss);

                if first.x == dimensions.x - 1 && first.y == dimensions.y - 1 {
                    return first.heat_loss.clone();
                } else {
                    let value = get_value(&first, grid, dimensions, &mut remaining_cache);
                    if value > max_val {
                        max_val = value;
                        println!(
                    "Path of len {} found, heat is {}, path left: {}/{}={}, value {}, HEADS:{}",
                    first.len,
                    first.heat_loss,
                    dimensions.x - first.x - 1,
                    dimensions.y - first.y - 1,
                    dimensions.x - first.x + dimensions.y - first.y - 2,
                    value,
                    remaining_items,
                );
                    } else if value < max_val {
                        println!("Did not expect decrease in value");
                    }
                    /*if min_path_left > dimensions.x - first.x + dimensions.y - first.y - 2 {
                        min_path_left = dimensions.x - first.x + dimensions.y - first.y - 2;
                        println!(
                            "Path of len {} found, heat is {}, path left: {}/{}={}, value {}",
                            first.len,
                            first.heat_loss,
                            dimensions.x - first.x - 1,
                            dimensions.y - first.y - 1,
                            dimensions.x - first.x + dimensions.y - first.y - 2,
                            first.heat_loss - first.x - first.y,
                        );
                    }*/

                    let (dir_1, dir_2, dir_3) = match first.current_direction {
                        Direction::Up => (Direction::Up, Direction::Left, Direction::Right),
                        Direction::Left => (Direction::Up, Direction::Left, Direction::Down),
                        Direction::Down => (Direction::Left, Direction::Down, Direction::Right),
                        Direction::Right => (Direction::Up, Direction::Down, Direction::Right),
                    };

                    match new_path_head(grid, &first, dir_1, dimensions) {
                        Some(mut head) => {
                            let opposing = match head.current_direction {
                                Direction::Up => Direction::Down,
                                Direction::Left => Direction::Right,
                                Direction::Down => Direction::Up,
                                Direction::Right => Direction::Left,
                            };
                            if head
                                .path
                                .iter()
                                .find(|(x, y, dir, len)| {
                                    *x == head.x
                                        && *y == head.y
                                        && ((*dir == head.current_direction
                                            && *len <= head.straight_len)
                                            || *dir != opposing)
                                })
                                .is_none()
                            {
                                head.path.push((
                                    head.x,
                                    head.y,
                                    head.current_direction,
                                    head.straight_len,
                                ));
                                let val = get_value(&head, grid, dimensions, &mut remaining_cache);
                                match explorations.get_mut(&val) {
                                    Some(list) => {
                                        if list
                                            .iter()
                                            .find(|head2| head.x == head2.x && head.y == head2.y)
                                            .is_none()
                                            && explored
                                                .iter()
                                                .find(|(x, y)| *x == head.x && *y == head.y)
                                                .is_none()
                                        {
                                            list.push(head);
                                        }
                                    }
                                    None => {
                                        explorations.insert(val, vec![head]);
                                    }
                                }
                            } else {
                            }
                        }
                        None => {}
                    }
                    match new_path_head(grid, &first, dir_2, dimensions) {
                        Some(mut head) => {
                            let opposing = match head.current_direction {
                                Direction::Up => Direction::Down,
                                Direction::Left => Direction::Right,
                                Direction::Down => Direction::Up,
                                Direction::Right => Direction::Left,
                            };
                            if head
                                .path
                                .iter()
                                .find(|(x, y, dir, len)| {
                                    *x == head.x
                                        && *y == head.y
                                        && ((*dir == head.current_direction
                                            && *len <= head.straight_len)
                                            || *dir != opposing)
                                })
                                .is_none()
                            {
                                head.path.push((
                                    head.x,
                                    head.y,
                                    head.current_direction,
                                    head.straight_len,
                                ));
                                let val = get_value(&head, grid, dimensions, &mut remaining_cache);
                                match explorations.get_mut(&val) {
                                    Some(list) => {
                                        if list
                                            .iter()
                                            .find(|head2| head.x == head2.x && head.y == head2.y)
                                            .is_none()
                                            && explored
                                                .iter()
                                                .find(|(x, y)| *x == head.x && *y == head.y)
                                                .is_none()
                                        {
                                            list.push(head);
                                        }
                                    }
                                    None => {
                                        explorations.insert(val, vec![head]);
                                    }
                                }
                            } else {
                            }
                        }
                        None => {}
                    }
                    match new_path_head(grid, &first, dir_3, dimensions) {
                        Some(mut head) => {
                            let opposing = match head.current_direction {
                                Direction::Up => Direction::Down,
                                Direction::Left => Direction::Right,
                                Direction::Down => Direction::Up,
                                Direction::Right => Direction::Left,
                            };
                            if head
                                .path
                                .iter()
                                .find(|(x, y, dir, len)| {
                                    *x == head.x
                                        && *y == head.y
                                        && ((*dir == head.current_direction
                                            && *len <= head.straight_len)
                                            || *dir != opposing)
                                })
                                .is_none()
                            {
                                head.path.push((
                                    head.x,
                                    head.y,
                                    head.current_direction,
                                    head.straight_len,
                                ));
                                let val = get_value(&head, grid, dimensions, &mut remaining_cache);
                                match explorations.get_mut(&val) {
                                    Some(list) => {
                                        if list
                                            .iter()
                                            .find(|head2| head.x == head2.x && head.y == head2.y)
                                            .is_none()
                                            && explored
                                                .iter()
                                                .find(|(x, y)| *x == head.x && *y == head.y)
                                                .is_none()
                                        {
                                            list.push(head);
                                        }
                                    }
                                    None => {
                                        explorations.insert(val, vec![head]);
                                    }
                                }
                            } else {
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {
                last_key += 1;
            }
        }
    }
}

fn get_node_dijkstra<'a>(
    grid: &'a mut Vec<DijkstraNode>,
    dimensions: &Dimension,
    x: usize,
    y: usize,
    straight_len: usize,
    direction: Direction,
) -> Option<&'a mut DijkstraNode> {
    return grid.get_mut(
        (x + dimensions.x * y) * 12
            + match direction {
                Direction::Left => 0,
                Direction::Right => 1,
                Direction::Up => 2,
                Direction::Down => 3,
            } * 3
            + (straight_len - 1),
    );
}
fn insert_new_exploration(
    explorations: &mut HashMap<usize, Vec<DijkstraFrom>>,
    exploration: DijkstraFrom,
) {
    match explorations.get_mut(&exploration.heat_loss) {
        Some(list) => {
            list.push(exploration);
        }
        None => {
            explorations.insert(exploration.heat_loss, vec![exploration]);
        }
    }
}

fn generate_next_exploration(
    dimensions: &Dimension,
    exploration: &DijkstraFrom,
    direction: Direction,
    x: usize,
    y: usize,
    node_heat: usize,
    reset_straight: bool,
) -> Option<DijkstraFrom> {
    match direction {
        Direction::Up => {
            if y == 0 {
                return None;
            }
        }
        Direction::Left => {
            if x == 0 {
                return None;
            }
        }
        Direction::Down => {
            if y >= dimensions.y - 1 {
                return None;
            }
        }
        Direction::Right => {
            if x >= dimensions.x - 1 {
                return None;
            }
        }
    }
    let new_exploration = DijkstraFrom {
        x: x,
        y: y,
        direction: direction,
        straight_len: if reset_straight {
            1
        } else {
            exploration.straight_len + 1
        },
        heat_loss: exploration.heat_loss + node_heat,
    };
    return Some(new_exploration);
}
fn find_path_dijkstra(grid: &mut Vec<Vec<Tile>>, dimensions: &Dimension) -> usize {
    let mut nodes: Vec<DijkstraNode> = grid
        .iter()
        .enumerate()
        .flat_map(|(i, line)| {
            line.iter().enumerate().flat_map(move |(j, tile)| {
                [
                    Direction::Left,
                    Direction::Right,
                    Direction::Up,
                    Direction::Down,
                ]
                .iter()
                .flat_map(|dir| {
                    [1, 2, 3].iter().map(|l| DijkstraNode {
                        heat_loss: tile.heat_loss,
                        x: j,
                        y: i,
                        prev_heat_loss: 0,
                        straight_len: *l,
                        direction: *dir,
                    })
                })
                .collect::<Vec<DijkstraNode>>()
            })
        })
        .collect();

    let start_a = DijkstraFrom {
        x: 0,
        y: 0,
        direction: Direction::Right,
        straight_len: 1,
        heat_loss: 0,
    };
    let start_b = DijkstraFrom {
        x: 0,
        y: 0,
        direction: Direction::Down,
        straight_len: 1,
        heat_loss: 0,
    };

    let mut explorations: HashMap<usize, Vec<DijkstraFrom>> = HashMap::new();
    explorations.insert(0, vec![start_a, start_b]);
    let mut last_key = 0;

    loop {
        let mut to_add: Vec<DijkstraFrom> = Vec::new();
        match explorations.get_mut(&last_key) {
            Some(list) => {
                let exploration = list.pop();

                match exploration {
                    Some(exploration) => {
                        let (mut x, mut y) = (exploration.x, exploration.y);
                        match exploration.direction {
                            Direction::Up => {
                                if y == 0 {
                                    panic!("invalid direction1")
                                } else {
                                    y -= 1;
                                }
                            }
                            Direction::Left => {
                                if x == 0 {
                                    panic!("invalid direction2")
                                } else {
                                    x -= 1;
                                }
                            }
                            Direction::Down => {
                                if y >= dimensions.y - 1 {
                                    panic!("invalid direction3")
                                } else {
                                    y += 1;
                                }
                            }
                            Direction::Right => {
                                if x >= dimensions.x - 1 {
                                    panic!("invalid direction4")
                                } else {
                                    x += 1;
                                }
                            }
                        }

                        match get_node_dijkstra(
                            &mut nodes,
                            dimensions,
                            x,
                            y,
                            exploration.straight_len,
                            exploration.direction,
                        ) {
                            Some(node) => {
                                /*if node
                                .from
                                .iter()
                                .find(|from| {
                                    from.heat_loss < exploration.heat_loss
                                        && from.straight_len <= exploration.straight_len
                                })
                                .is_none()*/
                                if node.prev_heat_loss == 0
                                    || node.prev_heat_loss > exploration.heat_loss
                                {
                                    //node.from.push(exploration.clone());
                                    node.prev_heat_loss = exploration.heat_loss;

                                    if x == dimensions.x - 1 && y == dimensions.y - 1 {
                                        return exploration.heat_loss + node.heat_loss;
                                    } else {
                                        if exploration.straight_len < 3 {
                                            let straight = generate_next_exploration(
                                                dimensions,
                                                &exploration,
                                                exploration.direction,
                                                x,
                                                y,
                                                node.heat_loss,
                                                false,
                                            );
                                            match straight {
                                                Some(new_exploration) => {
                                                    to_add.push(new_exploration)
                                                }
                                                None => {}
                                            }
                                        }

                                        let (left, right) = match exploration.direction {
                                            Direction::Up => (Direction::Left, Direction::Right),
                                            Direction::Left => (Direction::Down, Direction::Up),
                                            Direction::Down => (Direction::Right, Direction::Left),
                                            Direction::Right => (Direction::Up, Direction::Down),
                                        };
                                        let left_exploration = generate_next_exploration(
                                            dimensions,
                                            &exploration,
                                            left,
                                            x,
                                            y,
                                            node.heat_loss,
                                            true,
                                        );
                                        match left_exploration {
                                            Some(new_exploration) => to_add.push(new_exploration),
                                            None => {}
                                        }
                                        let right_exploration = generate_next_exploration(
                                            dimensions,
                                            &exploration,
                                            right,
                                            x,
                                            y,
                                            node.heat_loss,
                                            true,
                                        );
                                        match right_exploration {
                                            Some(new_exploration) => to_add.push(new_exploration),
                                            None => {}
                                        }
                                    }
                                }
                            }
                            None => println!("could not find node"),
                        }
                    }
                    None => {}
                }

                if list.len() == 0 {
                    explorations.remove(&last_key);
                    println!("Done: {}", last_key);
                    last_key += 1;
                }
            }
            None => {
                last_key += 1;
            }
        }
        for item in to_add {
            insert_new_exploration(&mut explorations, item);
        }
    }
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

            //let result = find_path(&mut grid, &dimensions);
            let result = find_path_dijkstra(&mut grid, &dimensions);

            println!("result is {}", result);
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
