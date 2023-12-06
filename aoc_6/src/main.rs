use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

fn analyze_number_list(parsed: Pair<'_, Rule>) -> Result<Vec<u32>, &'static str> {
    let mut list: Vec<u32> = Vec::new();
    for num in parsed.into_inner() {
        match num.as_rule() {
            Rule::number => match num.as_str().parse::<u32>() {
                Ok(i) => list.push(i),
                Err(e) => println!("Error parsing number {e}"),
            },
            Rule::EOI => {
                println!("    EOI {}", num.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(num list) {}", num.as_str());
            }
        }
    }
    return Ok(list);
}

fn analyze_number_list_entry(parsed: Pair<'_, Rule>) -> Result<Vec<u32>, &'static str> {
    let mut list: Option<Vec<u32>> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::number_list => match analyze_number_list(entry) {
                Ok(l) => list = Some(l),
                Err(_e) => {}
            },
            Rule::EOI => {
                println!("  EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(num list entry) {}", entry.as_str());
            }
        }
    }
    match list {
        Some(l) => return Ok(l),
        None => return Err("Found no number list"),
    }
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Result<(Vec<u32>, Vec<u32>), &'static str> {
    let mut times: Option<Vec<u32>> = None;
    let mut distances: Option<Vec<u32>> = None;
    let unwrapped = parsed.next().unwrap();
    for line in unwrapped.into_inner() {
        match line.as_rule() {
            Rule::times => match analyze_number_list_entry(line) {
                Ok(l) => times = Some(l),
                Err(_e) => {}
            },
            Rule::distances => match analyze_number_list_entry(line) {
                Ok(l) => distances = Some(l),
                Err(_e) => {}
            },
            Rule::EOI => {
                println!("EOI {}", line.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", line.as_str());
            }
        }
    }
    match (times, distances) {
        (Some(t), Some(d)) => return Ok((t, d)),
        (Some(_t), None) => return Err("Could not parse  distances"),
        (None, Some(_d)) => return Err("Could not parse times"),
        (None, None) => return Err("Could not parse times and distances"),
    }
}

fn restructure_races<'a>(times: &'a Vec<u32>, distances: &'a Vec<u32>) -> Vec<(&'a u32, &'a u32)> {
    let games: Vec<(&u32, &u32)> = times.iter().zip(distances.iter()).collect();
    return games;
}

fn count_win_possibilities(games: Vec<(&u32, &u32)>) -> Vec<u32> {
    return games
        .iter()
        .map(|&(time, distance)| {
            let mut wins: u32 = 0;
            for t0 in 1..*time {
                if (time - t0) * t0 >= *distance {
                    wins += 1;
                }
            }

            // t = t0 + d/t0
            // d = (t-t0)t0
            return wins;
        })
        .collect();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => match analyze_file(&mut result) {
            Ok((times, distances)) => {
                let games = restructure_races(&times, &distances);
                let wins = count_win_possibilities(games);
                let score = wins.iter().fold(1, |accumulator: u32, w| accumulator * w);

                println!("Score is {score}")
            }
            Err(e) => println!("Error parsing file: {e}"),
        },
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
