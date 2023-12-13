use std::cmp;
use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

#[derive(PartialEq, Clone, Copy, Eq)]
enum Tile {
    Ash,
    Rock,
}

struct Pattern {
    grid: Vec<Vec<Tile>>,
    reflection_x: Option<usize>,
    reflection_y: Option<usize>,
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Vec<Tile> {
    let mut line: Vec<Tile> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::ash => {
                line.push(Tile::Ash);
            }
            Rule::rock => {
                line.push(Tile::Rock);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return line;
}

fn analyze_pattern(parsed: Pair<'_, Rule>) -> Pattern {
    let mut grid: Vec<Vec<Tile>> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::line => {
                grid.push(analyze_line(entry));
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return Pattern {
        grid: grid,
        reflection_x: None,
        reflection_y: None,
    };
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<Pattern> {
    let mut patterns: Vec<Pattern> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::pattern => patterns.push(analyze_pattern(entry)),
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return patterns;
}

fn check_reflection_x(pattern: &Pattern, candidate_a: usize, candidate_b: usize) -> usize {
    let mut errors = 0;
    for line in &pattern.grid {
        let candidate_a = line.get(candidate_a).expect("Expected to find candidate_a");
        let candidate_b = line.get(candidate_b).expect("Expected to find candidate_a");
        if *candidate_a != *candidate_b {
            errors += 1;
            if errors > 1 {
                return errors;
            }
        }
    }
    return errors;
}
fn is_reflection_x(pattern: &Pattern, candidate: usize, len_x: usize) -> usize {
    let mut sum = 0;
    let to_check = cmp::min(candidate + 1, len_x - candidate - 1);
    for i in 0..to_check {
        sum += check_reflection_x(pattern, candidate - i, candidate + 1 + i);
    }
    return sum;
}
fn check_reflection_y(pattern: &Pattern, candidate_a: usize, candidate_b: usize) -> usize {
    let candidate_a = pattern
        .grid
        .get(candidate_a)
        .expect("Expected to find candidate_a");
    let candidate_b = pattern
        .grid
        .get(candidate_b)
        .expect("Expected to find candidate_a");

    let mut errors = 0;
    for (a, b) in candidate_a.iter().zip(candidate_b.iter()) {
        if *a != *b {
            errors += 1;
            if errors > 1 {
                return errors;
            }
        }
    }
    return errors;
}
fn is_reflection_y(pattern: &Pattern, candidate: usize, len_y: usize) -> usize {
    let mut sum = 0;
    let to_check = cmp::min(candidate + 1, len_y - candidate - 1);
    for i in 0..to_check {
        sum += check_reflection_y(pattern, candidate - i, candidate + 1 + i);
    }
    return sum;
}

fn find_reflections(patterns: &mut Vec<Pattern>) {
    for pattern in patterns.iter_mut() {
        let len_x = pattern
            .grid
            .first()
            .expect("expected at least one entry in grid")
            .len();
        let len_y = pattern.grid.len();
        for i in 0..len_x - 1 {
            if is_reflection_x(pattern, i, len_x) == 1 {
                pattern.reflection_x = Some(i);
                println!("Found reflection at X{i}");
                break;
            }
        }
        for i in 0..len_y - 1 {
            if is_reflection_y(pattern, i, len_y) == 1 {
                pattern.reflection_y = Some(i);
                println!("Found reflection at Y{i}");
                break;
            }
        }
    }
}

fn sum_reflections(patterns: &Vec<Pattern>) -> usize {
    let mut sum = 0;
    for pattern in patterns.iter() {
        let mut value = 0;
        match pattern.reflection_x {
            Some(x) => value += x + 1,
            None => {}
        }
        match pattern.reflection_y {
            Some(y) => value += 100 * (y + 1),
            None => {}
        }
        sum += value;
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
            let mut patterns = analyze_file(&mut result);
            find_reflections(&mut patterns);
            println!("Sum is {}", sum_reflections(&patterns));
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
