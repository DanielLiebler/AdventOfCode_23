use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use gcd::Gcd;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

enum Direction {
    Left,
    Right,
}
struct Node {
    ident: u32,
    left: u32,
    right: u32,
    fast_travel: u32,
}
impl Clone for Node {
    fn clone(&self) -> Self {
        return Node {
            ident: self.ident,
            left: self.left,
            right: self.right,
            fast_travel: self.fast_travel,
        };
    }
}
impl Copy for Node {}

fn ident_to_num(ident: &str) -> u32 {
    let mut chars = ident.chars();
    match (chars.next(), chars.next(), chars.next()) {
        (Some(c0), Some(c1), Some(c2)) => {
            return ((c0 as u32) << 16) + ((c1 as u32) << 8) + (c2 as u32);
        }
        _ => println!("Error converting ident"),
    }
    return 0;
}

fn analyze_node(parsed: Pair<'_, Rule>) -> Result<Node, &str> {
    let mut ident: Option<u32> = None;
    let mut left: Option<u32> = None;
    let mut right: Option<u32> = None;
    for node_entry in parsed.into_inner() {
        match node_entry.as_rule() {
            Rule::ident => ident = Some(ident_to_num(node_entry.as_str())),
            Rule::left => left = Some(ident_to_num(node_entry.as_str())),
            Rule::right => right = Some(ident_to_num(node_entry.as_str())),
            Rule::EOI => {
                println!("    EOI {}", node_entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(node) {}", node_entry.as_str());
            }
        }
    }
    match (ident, left, right) {
        (Some(i), Some(l), Some(r)) => {
            return Ok(Node {
                ident: i,
                left: l,
                right: r,
                fast_travel: i,
            })
        }
        _ => return Err("Error parsing node, not all components could be parsed"),
    }
}

fn analyze_sequence(parsed: Pair<'_, Rule>) -> Vec<Direction> {
    let mut seq: Vec<Direction> = Vec::new();
    for dir in parsed.into_inner() {
        match dir.as_rule() {
            Rule::direction => match dir.as_str().chars().nth(0) {
                Some('L') => seq.push(Direction::Left),
                Some('R') => seq.push(Direction::Right),
                Some(c) => println!("Char not a direction: {c}"),
                None => println!("Could not get first char of direction"),
            },
            Rule::EOI => {
                println!("    EOI {}", dir.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(seq) {}", dir.as_str());
            }
        }
    }
    return seq;
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Result<(Vec<Direction>, Vec<Node>), &'static str> {
    let mut seq: Option<Vec<Direction>> = None;
    let mut nodes: Vec<Node> = Vec::new();

    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::sequence => seq = Some(analyze_sequence(entry)),
            Rule::node => match analyze_node(entry) {
                Ok(node) => nodes.push(node),
                Err(e) => println!("Error parsing node {e}"),
            },
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    match seq {
        Some(s) => return Ok((s, nodes)),
        None => return Err("Could not parse sequence"),
    }
}

fn transmute_maze(seq: Vec<Direction>, nodes: &mut Vec<Node>) {
    let nodes_copy = nodes.to_vec();
    for dir in seq {
        for node in &mut *nodes {
            let fast_travel = nodes_copy
                .iter()
                .find(|n| n.ident == node.fast_travel)
                .expect("could not resolve pointer");
            let referenced = match &dir {
                Direction::Left => fast_travel.left,
                Direction::Right => fast_travel.right,
            };
            node.fast_travel = referenced;
        }
    }
}

fn traverse_maze(nodes: &Vec<Node>, steps_per_run: usize) -> usize {
    let mut node: &Node = nodes
        .iter()
        .find(|n| n.ident == ident_to_num("AAA"))
        .expect("could not find start");
    let mut runs = 0;
    loop {
        node = nodes
            .iter()
            .find(|n| n.ident == node.fast_travel)
            .expect("could not find start");
        runs += 1;
        if node.ident == ident_to_num("ZZZ") {
            break;
        }
    }

    return runs * steps_per_run;
}

fn check_node_type(ident: u32, to_check: &str) -> bool {
    let val = (ident ^ ident_to_num(to_check)) & 0xff == 0;
    return val;
}

fn to_ident(ident: u32) -> String {
    return String::from_utf8(vec![
        ((ident >> 16) & 0xff) as u8,
        ((ident >> 8) & 0xff) as u8,
        ((ident) & 0xff) as u8,
    ])
    .expect("could not convert back");
}

fn traverse_maze_ghost(nodes: &Vec<Node>, steps_per_run: usize) -> usize {
    let nodes_solved: Vec<(u32, usize, bool)> = nodes
        .iter()
        .map(|node| {
            (
                node.ident,
                nodes
                    .iter()
                    .enumerate()
                    .find_map(|(i, n)| {
                        if n.ident == node.fast_travel {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .expect("could not find start"),
                check_node_type(node.ident, "ZZZ"),
            )
        })
        .collect();
    let mut cur_nodes: Vec<(u32, usize, usize)> = nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| {
            if check_node_type(n.ident, "AAA") {
                Some((n.ident, i, i))
            } else {
                None
            }
        })
        .collect();
    /*println!("found {} starts", cur_nodes.len());
    let ends = nodes_solved.iter().filter(|(_ident, _refer, goal)| *goal);
    for (ident, _refer, goal) in ends {
        println!("Goal: {}({})", to_ident(*ident), *goal);
    }*/
    //println!("found {} ends", ends.count());

    let mut nodes_history: Vec<(Vec<(usize, bool)>, usize)> = cur_nodes
        .iter()
        .map(|(_ident, _start_refer, _refer)| (vec![(*_start_refer, false)], 0))
        .collect();

    let mut runs = 0;
    let mut finished = false;
    while !finished {
        /*if runs % 10000000 == 0 {
            println!("Round {}Mrd/13334", runs / 1000000000);
        }*/

        finished = true;
        for (i, (_start_ident, _start_index, node_index)) in cur_nodes.iter_mut().enumerate() {
            let referenced = nodes_solved.get(*node_index).expect("could not find node");

            let my_nodes_history = nodes_history.get_mut(i).expect("could not access history");
            match my_nodes_history
                .0
                .iter()
                .enumerate()
                .find(|(_i, &(j, _goal))| j == referenced.1)
            {
                Some((i, (_j, _goal))) => {
                    if my_nodes_history.1 == 0 {
                        my_nodes_history.1 = my_nodes_history.0.len() - i;
                        /*println!(
                            "Found circle for {} with length {}",
                            to_ident(*_start_ident),
                            my_nodes_history.1
                        );
                        for (h, _goal) in &my_nodes_history.0 {
                            print!(
                                "{} ",
                                to_ident(nodes_solved.get(*h).expect("could not solve pointer").0)
                            );
                        }
                        println!(
                            "{}",
                            to_ident(
                                nodes_solved
                                    .get(referenced.1)
                                    .expect("could not solve pointer")
                                    .0
                            )
                        );*/
                    }
                }
                None => {
                    my_nodes_history.0.push((
                        referenced.1,
                        nodes_solved
                            .get(referenced.1)
                            .expect("could not find node")
                            .2,
                    ));
                }
            }
            *node_index = referenced.1;

            if !nodes_solved
                .get(referenced.1)
                .expect("could not find node")
                .2
            {
                finished = false;
            }
        }
        if nodes_history.iter().filter(|(_h, len)| *len == 0).count() == 0 {
            break;
        }
        runs += 1;
    }
    /*println!("Runs: {runs}");

    print!("Goals per start loop: ");
    nodes_history
        .iter()
        .map(|(h, _len)| h.iter().filter(|(_refer, goal)| *goal).count())
        .for_each(|count| println!(" {count}"));
    println!("");*/

    let lcm_val = nodes_history
        .iter()
        .map(|(h, len)| {
            let index = h
                .iter()
                .enumerate()
                .filter_map(|(i, (_refer, goal))| if *goal { Some(i) } else { None })
                .next()
                .expect("expected a goal to exist");
            (index + len - h.len(), len, h.len() - len)
        })
        .fold(1, |accu, (index, len, skip)| {
            //println!("Goal at {skip}+{index}/{len}");
            return lcm(accu, *len);
        });
    //println!("Runs: {lcm_val}");
    runs = lcm_val;

    return runs * steps_per_run;
}

fn lcm(a: usize, b: usize) -> usize {
    return a * b / a.gcd(b);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let maze = analyze_file(&mut result);
            match maze {
                Ok((seq, mut nodes)) => {
                    let steps_per_run = seq.len();
                    transmute_maze(seq, &mut nodes);
                    let final_steps = 0; //traverse_maze(&nodes, steps_per_run);
                    let final_steps_ghost = traverse_maze_ghost(&nodes, steps_per_run);
                    println!("final count of steps is {final_steps}, {final_steps_ghost}");
                }
                Err(e) => println!("Error parsing maze: {e}"),
            }
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
