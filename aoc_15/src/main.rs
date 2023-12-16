use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

struct Lens {
    literal: String,
    hashed: usize,
    focal_len: usize,
}

fn hash_step(current_value: &usize, entry: char) -> usize {
    return ((*current_value + (entry as usize)) * 17) % 256;
}

fn add_entry(hashmap: &mut HashMap<usize, Vec<Lens>>, lens: Lens) {
    match hashmap.get_mut(&lens.hashed) {
        Some(entry) => match entry.iter_mut().find(|l| l.literal == lens.literal) {
            Some(l) => l.focal_len = lens.focal_len,
            None => entry.push(lens),
        },
        None => {
            hashmap.insert(lens.hashed, vec![lens]);
        }
    }
}

fn remove_entry(hashmap: &mut HashMap<usize, Vec<Lens>>, hashed: usize, literal: String) {
    match hashmap.get_mut(&hashed) {
        Some(entry) => match entry.iter().position(|l| l.literal == literal) {
            Some(p) => {
                entry.remove(p);
            }
            None => {
                println!(
                    "could not remove nonexistent lens, '{}'({})",
                    literal, hashed
                );
            }
        },
        None => {
            println!("trying to remove, but box does not exist")
        }
    }
}

fn analyze_hash(parsed: Pair<'_, Rule>) -> usize {
    let mut current_value: usize = 0;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::character => {
                let entry_char = entry.as_str().chars().next();
                match entry_char {
                    Some(',') => println!("Error: unexpected ,"),
                    Some('\n') => println!("Error: unexpected NEWLINE"),
                    Some(c) => current_value = hash_step(&current_value, c),
                    None => println!("Error: unexpected nothing"),
                }
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(hash) {}", entry.as_str());
            }
        }
    }
    return current_value;
}
fn analyze_add_entry(parsed: Pair<'_, Rule>, hashmap: &mut HashMap<usize, Vec<Lens>>) {
    let mut literal: Option<String> = None;
    let mut hashed: Option<usize> = None;
    let mut focal_len: Option<usize> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::hash => {
                literal = Some(entry.as_str().to_string());
                hashed = Some(analyze_hash(entry));
            }
            Rule::number => {
                focal_len = Some(
                    entry
                        .as_str()
                        .to_string()
                        .parse()
                        .expect("could not parse number"),
                )
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(add) {}", entry.as_str());
            }
        }
    }
    match (literal, hashed, focal_len) {
        (Some(literal), Some(hashed), Some(focal_len)) => add_entry(
            hashmap,
            Lens {
                literal: literal,
                hashed: hashed,
                focal_len: focal_len,
            },
        ),
        (_, _, _) => println!("Err, not all arguments for add supplied."),
    }
}
fn analyze_remove_entry(parsed: Pair<'_, Rule>, hashmap: &mut HashMap<usize, Vec<Lens>>) {
    let mut literal: Option<String> = None;
    let mut hashed: Option<usize> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::hash => {
                literal = Some(entry.as_str().to_string());
                hashed = Some(analyze_hash(entry));
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(remove) {}", entry.as_str());
            }
        }
    }
    match (literal, hashed) {
        (Some(literal), Some(hashed)) => remove_entry(hashmap, hashed, literal),
        (_, _) => println!("Err, not all arguments for add supplied."),
    }
}

fn analyze_entry(parsed: Pair<'_, Rule>, hashmap: &mut HashMap<usize, Vec<Lens>>) {
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::add_entry => analyze_add_entry(entry, hashmap),
            Rule::remove_entry => analyze_remove_entry(entry, hashmap),
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(entry) {}", entry.as_str());
            }
        }
    }
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> HashMap<usize, Vec<Lens>> {
    let mut hashmap: HashMap<usize, Vec<Lens>> = HashMap::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::entry => {
                analyze_entry(entry, &mut hashmap);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return hashmap;
}
fn get_focal_power(lens: &Lens, slot: usize) -> usize {
    return (lens.hashed + 1) * (slot + 1) * lens.focal_len;
}
fn sum_focal_power(hashmap: &HashMap<usize, Vec<Lens>>) -> usize {
    let mut sum = 0;
    for (_box_id, lenses) in hashmap.iter() {
        for (slot, lens) in lenses.iter().enumerate() {
            let focal_power = get_focal_power(lens, slot);
            sum += focal_power;
            println!(
                "Focal power of '{}'({}) @{} pow{} is {}",
                lens.literal, lens.hashed, slot, lens.focal_len, focal_power
            );
        }
    }
    return sum;
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let list = analyze_file(&mut result);
            let focal_power = sum_focal_power(&list);
            println!("focal power is {}", focal_power);
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
