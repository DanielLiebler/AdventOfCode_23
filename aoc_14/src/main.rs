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
fn tilt_south(grid: &mut Vec<Vec<Tile>>) {
    let line_width = grid.first().expect("Expected grid to contain items").len();
    let mut round_rocks_at_stop = vec![vec![(0, 0); 1]; line_width];
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
                Tile::Cube => {
                    round_rocks_at_stop
                        .last_mut()
                        .expect("expect to find an entry")
                        .0 = if i == 0 { 0 } else { i - 1 };
                    round_rocks_at_stop.push((0, 0));
                }
            }
        }
    }
    for round_rocks_at_stop in round_rocks_at_stop.iter_mut() {
        round_rocks_at_stop
            .last_mut()
            .expect("expect to find an entry")
            .0 = grid.len() - 1;
    }
    for (i, line) in grid.iter_mut().enumerate() {
        for (tile, round_rocks_at_stop) in line.iter_mut().zip(round_rocks_at_stop.iter()) {
            if round_rocks_at_stop
                .iter()
                .find(|(start, count)| i >= *start + 1 - *count && i <= *start)
                .is_some()
            {
                *tile = Tile::Round;
            }
        }
    }
}
fn tilt_west(grid: &mut Vec<Vec<Tile>>) {
    for line in grid.iter_mut() {
        let mut round_rocks_at_stop = vec![(0, 0); 1];
        for (j, tile) in line.iter_mut().enumerate() {
            match tile {
                Tile::Empty => {}
                Tile::Round => {
                    *tile = Tile::Empty;
                    round_rocks_at_stop
                        .last_mut()
                        .expect("expect to find an entry")
                        .1 += 1;
                }
                Tile::Cube => round_rocks_at_stop.push((j + 1, 0)),
            }
        }
        for (j, tile) in line.iter_mut().enumerate() {
            if round_rocks_at_stop
                .iter()
                .find(|(start, count)| j >= *start && j < *start + *count)
                .is_some()
            {
                *tile = Tile::Round;
            }
        }
    }
}
fn tilt_east(grid: &mut Vec<Vec<Tile>>) {
    for line in grid.iter_mut() {
        let mut round_rocks_at_stop = vec![(0, 0); 1];
        for (j, tile) in line.iter_mut().enumerate() {
            match tile {
                Tile::Empty => {}
                Tile::Round => {
                    *tile = Tile::Empty;
                    round_rocks_at_stop
                        .last_mut()
                        .expect("expect to find an entry")
                        .1 += 1;
                }
                Tile::Cube => {
                    round_rocks_at_stop
                        .last_mut()
                        .expect("expect to find an entry")
                        .0 = if j == 0 { 0 } else { j - 1 };
                    round_rocks_at_stop.push((0, 0));
                }
            }
        }
        round_rocks_at_stop
            .last_mut()
            .expect("expect to find an entry")
            .0 = line.len() - 1;

        for (j, tile) in line.iter_mut().enumerate() {
            if round_rocks_at_stop
                .iter()
                .find(|(start, count)| j >= *start + 1 - *count && j <= *start)
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

fn grid_as_nums(grid: &Vec<Vec<Tile>>) -> Vec<u128> {
    return grid
        .iter()
        .map(|line| {
            // 100x100
            line.iter().fold(0, |accu: u128, item| {
                accu * 2
                    + match item {
                        Tile::Empty => 0,
                        Tile::Round => 1,
                        Tile::Cube => 0,
                    }
            })
        })
        .collect();
}

fn cycle(grid: &mut Vec<Vec<Tile>>, count: usize) {
    let mut cache: HashMap<Vec<u128>, usize> = HashMap::new();

    cache.insert(grid_as_nums(&grid), 0);
    for _i in 0..count {
        if _i % 1000 == 0 {
            println!("Cycle{}k/{}k", _i / 1000, count / 1000);
        }
        tilt_north(grid);
        tilt_west(grid);
        tilt_south(grid);
        tilt_east(grid);

        let grid_id = grid_as_nums(&grid);
        match cache.get(&grid_id) {
            Some(cached) => {
                println!("FoundMatch: {} at position {}", *cached, _i + 1);
                //Contains a repeating pattern
                let pattern_len = _i + 1 - *cached;
                let remaining = (count - *cached) % pattern_len;

                if remaining > 0 {
                    for _j in 0..remaining {
                        tilt_north(grid);
                        tilt_west(grid);
                        tilt_south(grid);
                        tilt_east(grid);
                    }
                }

                return;
            }
            None => {
                cache.insert(grid_id, _i + 1);
            }
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

            cycle(&mut grid, 1000000000);
            println!("1000000000 Cycle");
            _print_grid(&grid);
            let sum = calculate_load(&grid);
            println!("Sum is {sum}")
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
