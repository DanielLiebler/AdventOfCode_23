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
    Empty,
    Round,
    Cube,
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Vec<Tile> {
    let mut line: Vec<Tile> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::empty => {
                line.push(Tile::Empty);
            }
            Rule::round => {
                line.push(Tile::Round);
            }
            Rule::cube => {
                line.push(Tile::Cube);
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

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<Vec<Tile>> {
    let mut grid: Vec<Vec<Tile>> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::line => grid.push(analyze_line(entry)),
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

fn tilt_north(grid: &mut Vec<Vec<Tile>>) {
    let mut round_rocks_at_stop =
        vec![vec![(0, 0); 1]; grid.first().expect("Expected grid to contain items").len()];
    for (i, line) in grid.iter_mut().enumerate() {
        for (tile, round_rocks_at_stop) in line.iter_mut().zip(round_rocks_at_stop.iter_mut()) {
            match tile {
                Tile::Empty => {}
                Tile::Round => {
                    *tile = Tile::Empty;
                    round_rocks_at_stop
                        .last_mut()
                        .expect("expect to find an entry")
                        .1 += 1;
                }
                Tile::Cube => round_rocks_at_stop.push((i + 1, 0)),
            }
        }
    }
    for (i, line) in grid.iter_mut().enumerate() {
        for (tile, round_rocks_at_stop) in line.iter_mut().zip(round_rocks_at_stop.iter()) {
            if round_rocks_at_stop
                .iter()
                .find(|(start, count)| i >= *start && i < *start + *count)
                .is_some()
            {
                *tile = Tile::Round;
            }
        }
    }
}
fn calculate_load(grid: &Vec<Vec<Tile>>) -> usize {
    let rows = grid.len();
    let mut sum = 0;
    for (i, line) in grid.iter().enumerate() {
        for tile in line.iter() {
            if *tile == Tile::Round {
                sum += rows - i;
            }
        }
    }
    return sum;
}

fn _print_grid(grid: &Vec<Vec<Tile>>) {
    for line in grid.iter() {
        for tile in line.iter() {
            print!(
                "{}",
                match tile {
                    Tile::Empty => ".",
                    Tile::Round => "O",
                    Tile::Cube => "#",
                }
            );
        }
        println!();
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
            tilt_north(&mut grid);
            _print_grid(&grid);
            let sum = calculate_load(&grid);
            println!("Sum is {sum}")
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
