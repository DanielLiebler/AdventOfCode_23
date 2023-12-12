use std::cmp;
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
enum Spring {
    Operational,
    Damaged,
    Unknown,
}

struct Line {
    spring_list: Vec<Spring>,
    number_list: Vec<usize>,
    arrangements: usize,
}

fn analyze_num_list(parsed: Pair<'_, Rule>) -> Vec<usize> {
    let mut list: Vec<usize> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::num => list.push(
                entry
                    .as_str()
                    .parse()
                    .expect("could not parse sequence entry"),
            ),
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return list;
}
fn analyze_spring(parsed: Pair<'_, Rule>) -> Spring {
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::operational => {
                return Spring::Operational;
            }
            Rule::damaged => {
                return Spring::Damaged;
            }
            Rule::unknown => {
                return Spring::Unknown;
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return Spring::Unknown;
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Result<Line, &'static str> {
    let mut springs: Vec<Spring> = Vec::new();
    let mut num_list: Option<Vec<usize>> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::spring => {
                springs.push(analyze_spring(entry));
            }
            Rule::spring_list => {
                num_list = Some(analyze_num_list(entry));
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return match num_list {
        Some(list) => Ok(Line {
            spring_list: springs,
            number_list: list,
            arrangements: 0,
        }),
        None => Err("Did not find numerical list"),
    };
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<Line> {
    let mut space: Vec<Line> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::line => match analyze_line(entry) {
                Ok(l) => space.push(l),
                Err(_e) => println!("Error: {_e}"),
            },
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

fn find_arrangements_in_block(
    (len, block): (usize, &[Spring]),
    pattern: &Vec<usize>,
    start: usize,
    pattern_count: usize,
) -> usize {
    //print!("\n        Block: S{start}({pattern_count} blks) strlen[{len}]");
    if !match pattern.get(start..start + pattern_count) {
        Some(slice) => slice.iter().fold(0, |accu, l| accu + *l + 1) <= len + 1,
        None => false,
    } {
        // there is not enough space to fit the patterns begin with
        return 0;
    }

    let max_skip =
        match block
            .iter()
            .enumerate()
            .find_map(|(i, s)| if *s == Spring::Unknown { None } else { Some(i) })
        {
            Some(i) => i,
            None => block.len(),
        };
    let first_pattern = pattern.get(start).expect("could not find first pattern");
    if pattern_count == 1 {
        return (*first_pattern..cmp::min(*first_pattern + max_skip + 1, len + 1))
            .filter(|i| {
                block
                    .get(*i..)
                    .expect("Expected to find some")
                    .iter()
                    .filter(|&s| *s == Spring::Damaged)
                    .count()
                    == 0
            })
            .count();
    } else {
        let mut arrangements = 0;
        for skip in *first_pattern + 1..*first_pattern + 1 + max_skip + 1 {
            if skip >= len {
                break;
            }
            if *block.get(skip - 1).expect("could not get remaining block") == Spring::Damaged {
                // cant use this as a break
                continue;
            }
            arrangements += find_arrangements_in_block(
                (
                    len - skip,
                    block.get(skip..).expect("could not get remaining block"),
                ),
                pattern,
                start + 1,
                pattern_count - 1,
            );
        }
        return arrangements;
    }
}

fn find_fitting_blocks(
    (len, block): (usize, &[Spring]),
    pattern: &Vec<usize>,
    proposed_starts: impl Iterator<Item = usize>,
) -> Vec<Vec<(usize, usize)>> {
    print!("  Finding Fitting blocks in block({len}):");

    let mut total_arrangements: Vec<Vec<(usize, usize)>> = Vec::new();
    for start in proposed_starts {
        print!("\n    S {start}:");
        let mut block_count: usize = 1;
        let mut arrangements: Vec<(usize, usize)> = Vec::new();

        // check if not matching any is an option
        match block.iter().find(|s| **s == Spring::Damaged) {
            Some(_) => {}
            None => arrangements.push((0, 1)),
        }

        loop {
            let arrangements_in_block =
                find_arrangements_in_block((len, block), pattern, start, block_count);
            if arrangements_in_block == 0 {
                // no arrangements found, we can skip searching
                //break;
                if start + block_count >= pattern.len() {
                    break;
                }
                block_count += 1;
            } else {
                print!("\n      {block_count}({arrangements_in_block})");
                arrangements.push((block_count, arrangements_in_block));
                block_count += 1;
            }
        }
        total_arrangements.push(arrangements);
    }
    println!();
    return total_arrangements;
}

fn find_arrangements_in_line(line: &Line, line_num: usize) -> usize {
    let mut max_lists: Vec<(usize, &[Spring])> = Vec::new();

    let mut count = 0;
    for (i, spring) in line.spring_list.iter().enumerate() {
        if *spring == Spring::Operational {
            if count != 0 {
                max_lists.push((
                    count,
                    line.spring_list
                        .get(i - count..i)
                        .expect("could not slice spring_list"),
                ));
                count = 0;
            }
        } else {
            count += 1;
        }
    }
    if count != 0 {
        max_lists.push((
            count,
            line.spring_list
                .get(line.spring_list.len() - count..)
                .expect("could not slice spring_list"),
        ));
    }

    print!("MaxList({}):", line_num);
    for block in &max_lists {
        print!(" {}", block.0);
    }
    println!();

    let mut starts: Vec<(usize, usize)> = vec![(0, 1)];
    for block in &max_lists {
        // find amount of blocks able to fit here
        let arrs = find_fitting_blocks(
            *block,
            &line.number_list,
            starts.iter().map(|(start, _arrs)| *start),
        );
        starts = arrs
            .iter()
            .zip(starts.iter())
            .flat_map(|(new_arrangements, (start, total_arrangements))| {
                let n_a: Vec<(usize, usize)> = new_arrangements
                    .iter()
                    .map(|(block_count, arrangements_in_block)| {
                        (
                            start + block_count,
                            total_arrangements * *arrangements_in_block,
                        )
                    })
                    .collect();
                return n_a;
            })
            .collect();
        starts = starts.iter().fold(Vec::new(), |accu, item| {
            let mut accu = accu.clone();
            match accu.iter_mut().find(|i| i.0 == item.0) {
                Some(i) => {
                    i.1 += item.1;
                    return accu;
                }
                None => {
                    accu.push(item.clone());
                    return accu;
                }
            }
        });
    }

    return starts.iter().fold(0, |accu, (start, arr)| {
        if *start == line.number_list.len() {
            accu + *arr
        } else {
            accu
        }
    });
}

fn find_arrangements(lines: &mut Vec<Line>) {
    let mut sum = 0;
    for (i, line) in lines.iter_mut().enumerate() {
        line.arrangements = find_arrangements_in_line(line, i);
        println!("--> Line {i}: {}\n\n", line.arrangements);
        sum += line.arrangements;
    }
    println!("Sum is {sum}");
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let mut lines = analyze_file(&mut result);
            find_arrangements(&mut lines);
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
