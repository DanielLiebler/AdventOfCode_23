use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use colored::Colorize;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

#[derive(PartialEq, Clone, Copy)]
enum Space {
    Space,
    Galaxy,
}
#[derive(PartialEq, Clone, Copy)]
struct Coordinates {
    x: usize,
    y: usize,
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Vec<Space> {
    let mut space: Vec<Space> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::empty => {
                space.push(Space::Space);
            }
            Rule::galaxy => {
                space.push(Space::Galaxy);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return space;
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<Vec<Space>> {
    let mut space: Vec<Vec<Space>> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::line => space.push(analyze_line(entry)),
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return space;
}

fn find_galaxies(space: &Vec<Vec<Space>>) -> (Vec<Coordinates>, Vec<usize>, Vec<usize>) {
    let mut galaxies: Vec<Coordinates> = Vec::new();

    let dim_x = space.first().expect("No Space supplied").len();
    let dim_y = space.len();

    let mut expanding_x: Vec<bool> = vec![true; dim_x];
    let mut expanding_y: Vec<bool> = vec![true; dim_y];

    for (y, line) in space.iter().enumerate() {
        for (x, location) in line.iter().enumerate() {
            if *location == Space::Galaxy {
                galaxies.push(Coordinates { x: x, y: y });

                *expanding_x
                    .get_mut(x)
                    .expect("Could not access item of expanding_x") = false;
                *expanding_y
                    .get_mut(y)
                    .expect("Could not access item of expanding_y") = false;
            }
        }
    }

    let expanding_x: Vec<usize> = expanding_x
        .iter()
        .enumerate()
        .filter_map(|(i, &is_empty)| {
            if is_empty {
                return Some(i);
            }
            return None;
        })
        .collect();
    let expanding_y: Vec<usize> = expanding_y
        .iter()
        .enumerate()
        .filter_map(|(i, &is_empty)| {
            if is_empty {
                return Some(i);
            }
            return None;
        })
        .collect();

    return (galaxies, expanding_x, expanding_y);
}

fn order_coordinates(a: &Coordinates, b: &Coordinates) -> ((usize, usize), (usize, usize)) {
    let x = if a.x <= b.x { (a.x, b.x) } else { (b.x, a.x) };

    let y = if a.y <= b.y { (a.y, b.y) } else { (b.y, a.y) };
    return (x, y);
}
fn count_expansions(low: usize, high: usize, expanding: &Vec<usize>) -> usize {
    expanding.iter().filter(|&i| *i > low && *i < high).count()
}
fn count_path_len(low: usize, high: usize, expanding: &Vec<usize>) -> usize {
    let expansions_x = count_expansions(low, high, expanding);
    return high - low + expansions_x;
}
fn find_distance(
    a: &Coordinates,
    b: &Coordinates,
    expanding_x: &Vec<usize>,
    expanding_y: &Vec<usize>,
) -> usize {
    let ((x_l, x_h), (y_l, y_h)) = order_coordinates(a, b);

    let path_len_x = count_path_len(x_l, x_h, expanding_x);
    let path_len_y = count_path_len(y_l, y_h, expanding_y);

    return path_len_x + path_len_y;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let space = analyze_file(&mut result);
            let (galaxies, expanding_x, expanding_y) = find_galaxies(&space);

            let mut total_distance = 0;

            for (i, galaxy_a) in galaxies.iter().enumerate() {
                for (j, galaxy_b) in galaxies.iter().enumerate() {
                    if j <= i {
                        continue;
                    }
                    let distance = find_distance(galaxy_a, galaxy_b, &expanding_x, &expanding_y);
                    println!(
                        "G{i}, G{j}: {distance}  ({}/{} {}/{})",
                        galaxy_a.x, galaxy_a.y, galaxy_b.x, galaxy_b.y
                    );
                    total_distance += distance;
                }
            }
            println!("Total distance is {total_distance}");
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
