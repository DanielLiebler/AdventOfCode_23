use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

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
                    let final_steps = traverse_maze(&nodes, steps_per_run);
                    println!("final count of steps is {final_steps}");
                }
                Err(e) => println!("Error parsing maze: {e}"),
            }
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
