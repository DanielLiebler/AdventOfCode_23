use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

fn new_entry(current_value: &u128, entry: char) -> u128 {
    return ((*current_value + (entry as u128)) * 17) % 256;
}

fn analyze_entry(parsed: Pair<'_, Rule>) -> u128 {
    let mut current_value: u128 = 0;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::character => {
                let entry_char = entry.as_str().chars().next();
                match entry_char {
                    Some(',') => println!("Error: unexpected ,"),
                    Some('\n') => println!("Error: unexpected NEWLINE"),
                    Some(c) => current_value = new_entry(&current_value, c),
                    None => println!("Error: unexpected nothing"),
                }
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return current_value;
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<u128> {
    let mut list: Vec<u128> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::entry => {
                let str = entry.as_str();
                let value = analyze_entry(entry);
                list.push(value);
                println!("Value is {} for '{}'", value, str);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return list;
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let list = analyze_file(&mut result);
            println!("Sum is {}", list.iter().fold(0, |accu, i| accu + i));
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
