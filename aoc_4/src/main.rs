use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "scratchcards.pest"]
struct MyParser;

fn analyze_list(list: Pair<'_, Rule>) -> Vec<u32> {
    let mut parsed: Vec<u32> = Vec::new();

    for num in list.into_inner() {
        match num.as_rule() {
            Rule::number => {
                let i: u32 = num.as_str().parse().expect("unable to parse number");
                parsed.push(i);
            }
            Rule::EOI => {
                println!("   EOI {}", num.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(List) {}", num.as_str());
            }
        }
    }

    parsed.sort();
    return parsed;
}

fn analyze_line(line: Pair<'_, Rule>) -> u32 {
    let mut win_list: Option<Vec<u32>> = None;
    let mut num_list: Option<Vec<u32>> = None;
    for entry in line.into_inner() {
        match entry.as_rule() {
            Rule::winning_list => {
                println!("   Wins {}", entry.as_str());
                win_list = Some(analyze_list(entry));
            }
            Rule::number_list => {
                println!("   Nums {}", entry.as_str());
                num_list = Some(analyze_list(entry));
            }
            Rule::EOI => {
                println!(" EOI {}", entry.as_str());
            }
            Rule::game_identifier => {
                println!(" Card {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(Line) {}", entry.as_str());
            }
        }
    }

    match (win_list, num_list) {
        (Some(wins), Some(nums)) => {
            let mut points: u32 = 0;
            for num in nums {
                if wins.contains(&num) {
                    if points == 0 {
                        points = 1;
                    } else {
                        points *= 2;
                    }
                }
            }
            return points;
        }
        (None, Some(_)) => {
            println!("wins were not found");
        }
        (Some(_), None) => {
            println!("nums were not found");
        }
        (None, None) => {
            println!("wins and nums were not found");
        }
    }
    return 0;
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> u32 {
    let mut sum = 0;

    let unwrapped = parsed.next().unwrap();
    for line in unwrapped.into_inner() {
        match line.as_rule() {
            Rule::line => {
                sum += analyze_line(line);
            }
            Rule::EOI => {
                println!("EOI {}", line.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", line.as_str());
            }
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
            println!("Parse file OK");
            println!("==========");
            let sum = analyze_file(&mut result);
            println!("==========");
            println!("Points: {sum}");
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
