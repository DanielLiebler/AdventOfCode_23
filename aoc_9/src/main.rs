use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

struct Report {
    sequence: Vec<i32>,
    deductions: Vec<Vec<i32>>,
    next_val: i32,
}

fn analyze_report(parsed: Pair<'_, Rule>) -> Report {
    let mut sequence: Vec<i32> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::entry => {
                sequence.push(
                    entry
                        .as_str()
                        .parse()
                        .expect("could not parse sequence entry"),
                );
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(report) {}", entry.as_str());
            }
        }
    }
    return Report {
        sequence: sequence,
        deductions: Vec::new(),
        next_val: 0,
    };
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<Report> {
    let mut reports: Vec<Report> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::report => reports.push(analyze_report(entry)),
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return reports;
}

fn only_zeroes(vec: &Vec<i32>) -> bool {
    return vec.iter().filter(|&item| *item != 0).count() == 0;
}

fn find_deductions(reports: &mut Vec<Report>) {
    for report in reports.iter_mut() {
        if (report.deductions.len() == 0 && only_zeroes(&report.sequence))
            || (report.deductions.len() != 0
                && only_zeroes(
                    report
                        .deductions
                        .last()
                        .expect("could not access last deduction1"),
                ))
        {
        } else {
            loop {
                let last_deduction = if report.deductions.len() == 0 {
                    &report.sequence
                } else {
                    report
                        .deductions
                        .last()
                        .expect("could not access last deduction2")
                };
                let new_deduction = last_deduction
                    .iter()
                    .zip(last_deduction.iter().skip(1))
                    .map(|(&a, &b)| b - a)
                    .collect();
                let is_finished = only_zeroes(&new_deduction);
                report.deductions.push(new_deduction);
                if is_finished {
                    break;
                }
            }
        }
    }
}

fn extrapolate_reports(reports: &mut Vec<Report>) {
    for report in reports.iter_mut() {
        if report.deductions.len() == 0 {
            report.next_val = *report
                .sequence
                .last()
                .expect("could not access last of sequence");
            report.sequence.push(report.next_val);
        } else {
            report.deductions.reverse();
            let mut last_item = 0;
            report
                .deductions
                .get_mut(0)
                .expect("could not slice deductions(0)")
                .push(0);
            for deduction in report
                .deductions
                .get_mut(1..)
                .expect("Could not slice deductions")
                .iter_mut()
            {
                last_item = deduction
                    .last()
                    .expect("could not access last item of deduction")
                    + last_item;
                deduction.push(last_item);
            }
            last_item = report
                .sequence
                .last()
                .expect("could not access last item of deduction")
                + last_item;
            report.next_val = last_item;
            report.sequence.push(report.next_val);
        }
    }
}

fn sum_extrapolations(reports: &Vec<Report>) -> i32 {
    return reports.iter().fold(0, |accu, rep| {
        //println!("Summing: {}", rep.next_val);
        accu + rep.next_val
    });
}

fn _print_report(reports: &Vec<Report>) {
    for rep in reports {
        print!("Report:");
        for seq in &rep.sequence {
            print!(" {seq}");
        }
        println!("");
        for (i, ded) in rep.deductions.iter().enumerate() {
            print!("Deduction{i}:");
            for j in ded {
                print!(" {}", *j);
            }
            println!("");
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
            let mut reports = analyze_file(&mut result);
            find_deductions(&mut reports);
            extrapolate_reports(&mut reports);

            //_print_report(&reports);

            let sum = sum_extrapolations(&reports);
            println!("Sum is {sum}");
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
