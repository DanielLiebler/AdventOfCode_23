use std::cmp::max;
use std::cmp::min;
use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

struct WorkflowRule {
    variable: char,
    less_than: Option<usize>,
    greater_than: Option<usize>,
    target: Target,
}

#[derive(PartialEq, Clone, Eq)]
enum Target {
    Accept,
    Reject,
    Target(String),
}

struct Workflow {
    ident: String,
    rules: Vec<WorkflowRule>,
    default: Target,
}

struct Variable {
    ident: char,
    value: usize,
}
struct Part {
    current_step: Target,
    variables: Vec<Variable>,
}

#[derive(PartialEq, Clone, Eq)]
struct Range {
    from: usize,
    to: usize,
}
#[derive(PartialEq, Clone, Eq)]
struct Ranges {
    x: Range,
    m: Range,
    a: Range,
    s: Range,
}
struct Exploration {
    target: Target,
    ranges: Ranges,
}

fn analyze_variable(parsed: Pair<'_, Rule>) -> Result<Variable, &'static str> {
    let mut ident: Option<char> = None;
    let mut value: Option<usize> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::variable => {
                ident = Some(
                    entry
                        .as_str()
                        .chars()
                        .next()
                        .expect("Expected variable name"),
                );
            }
            Rule::number => {
                value = Some(entry.as_str().parse().expect("Could not parse variable"));
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(workflow_rule) {}", entry.as_str());
            }
        }
    }
    match (ident, value) {
        (Some(i), Some(v)) => return Ok(Variable { ident: i, value: v }),
        (_, _) => return Err("Could not parse variable"),
    }
}
fn analyze_parts(parsed: Pair<'_, Rule>) -> Result<Part, &'static str> {
    let mut variables: Vec<Variable> = Vec::new();
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::variable_entry => match analyze_variable(entry) {
                Ok(v) => variables.push(v),
                Err(_e) => return Err(_e),
            },
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(workflow_rule) {}", entry.as_str());
            }
        }
    }
    if variables.len() > 0 {
        return Ok(Part {
            current_step: Target::Target(String::from("in")),
            variables: variables,
        });
    }
    return Err("()");
}
fn analyze_ident(parsed: Pair<'_, Rule>) -> Result<Target, &'static str> {
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::workflow_ident => {
                return Ok(Target::Target(entry.as_str().to_string()));
            }
            Rule::accept => {
                return Ok(Target::Accept);
            }
            Rule::reject => {
                return Ok(Target::Reject);
            }
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(workflow_rule) {}", entry.as_str());
            }
        }
    }
    return Err("Could not parse Ident");
}
fn analyze_workflow_rule(parsed: Pair<'_, Rule>) -> Result<WorkflowRule, &'static str> {
    let mut var: Option<char> = None;
    let mut less: Option<bool> = None;
    let mut value: Option<usize> = None;
    let mut target: Option<Target> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::variable => {
                var = Some(
                    entry
                        .as_str()
                        .chars()
                        .next()
                        .expect("Could not parse variable"),
                );
            }
            Rule::greater => {
                less = Some(false);
            }
            Rule::less => {
                less = Some(true);
            }
            Rule::number => {
                value = Some(
                    entry
                        .as_str()
                        .parse()
                        .expect("could not parse value for rule"),
                );
            }
            Rule::ident => match analyze_ident(entry) {
                Ok(t) => target = Some(t),
                Err(_e) => println!("Error parsing target: {_e}"),
            },
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(workflow_rule) {}", entry.as_str());
            }
        }
    }
    match (var, less, value, target) {
        (Some(var), Some(less), Some(value), Some(target)) => {
            return Ok(WorkflowRule {
                variable: var,
                less_than: if less { Some(value) } else { None },
                greater_than: if !less { Some(value) } else { None },
                target,
            });
        }
        (_, _, _, _) => return Err("Could not parse rule"),
    }
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Result<Workflow, &'static str> {
    let mut workflow_ident: Option<String> = None;
    let mut rules: Vec<WorkflowRule> = Vec::new();
    let mut default_target: Option<Target> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::workflow_ident => {
                workflow_ident = Some(entry.as_str().to_string());
            }
            Rule::rule => match analyze_workflow_rule(entry) {
                Ok(rule) => rules.push(rule),
                Err(_e) => println!("Could not parse rule: {_e}"),
            },
            Rule::ident => match analyze_ident(entry) {
                Ok(t) => default_target = Some(t),
                Err(_e) => println!("Error parsing target: {_e}"),
            },
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(line) {}", entry.as_str());
            }
        }
    }

    //match (direction, count) {
    match (workflow_ident, default_target) {
        (Some(ident), Some(default)) => {
            return Ok(Workflow {
                ident,
                rules,
                default,
            })
        }
        (_, _) => return Err("Error parsing line"),
    }
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> (Vec<Workflow>, Vec<Part>) {
    let mut workflows: Vec<Workflow> = Vec::new();
    let mut parts: Vec<Part> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for entry in unwrapped.into_inner() {
        match entry.as_rule() {
            Rule::line => match analyze_line(entry) {
                Ok(entry) => workflows.push(entry),
                Err(_e) => println!("Error: {}", _e),
            },
            Rule::entry => match analyze_parts(entry) {
                Ok(entry) => parts.push(entry),
                Err(_e) => println!("Error: {}", _e),
            },
            Rule::EOI => {
                println!("EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", entry.as_str());
            }
        }
    }
    return (workflows, parts);
}

fn rule_matches(rule: &WorkflowRule, part: &Part) -> bool {
    match part
        .variables
        .iter()
        .find(|variable| variable.ident == rule.variable)
    {
        Some(variable) => match (rule.greater_than, rule.less_than) {
            (None, None) => return true,
            (None, Some(lt)) => return variable.value < lt,
            (Some(gt), None) => return variable.value > gt,
            (Some(gt), Some(lt)) => return variable.value < lt && variable.value > gt,
        },
        None => return false,
    }
}
fn run_workflow(workflows: &Vec<Workflow>, part: &mut Part) {
    print!("    Part: ");
    for val in part.variables.iter() {
        print!("{}:{} \t", val.ident, val.value);
    }
    println!();
    loop {
        match &part.current_step {
            Target::Accept => break,
            Target::Reject => break,
            Target::Target(step) => {
                match workflows.iter().find(|workflow| workflow.ident == *step) {
                    Some(workflow) => {
                        part.current_step =
                            match workflow.rules.iter().find(|rule| rule_matches(*rule, part)) {
                                Some(rule) => rule.target.clone(),
                                None => workflow.default.clone(),
                            };
                        match &part.current_step {
                            Target::Accept => println!("      A"),
                            Target::Reject => println!("      R"),
                            Target::Target(t) => println!("      {t}"),
                        }
                    }
                    None => panic!("Could not find Workflow"),
                }
            }
        }
    }
}

fn constrict_range(range: &Range, rule: &WorkflowRule) -> Range {
    match (rule.less_than, rule.greater_than) {
        (None, None) => return range.clone(),
        (None, Some(gt)) => {
            return Range {
                from: max(range.from, gt + 1),
                to: range.to,
            }
        }
        (Some(lt), None) => {
            return Range {
                from: range.from,
                to: min(range.to, lt - 1),
            }
        }
        (Some(lt), Some(gt)) => {
            return Range {
                from: max(range.from, gt + 1),
                to: min(range.to, lt - 1),
            }
        }
    }
}
fn constrict_ranges(ranges: &Ranges, rule: &WorkflowRule) -> Ranges {
    match rule.variable {
        'x' => {
            return Ranges {
                x: constrict_range(&ranges.x, rule),
                m: ranges.m.clone(),
                a: ranges.a.clone(),
                s: ranges.s.clone(),
            };
        }
        'm' => {
            return Ranges {
                x: ranges.x.clone(),
                m: constrict_range(&ranges.m, rule),
                a: ranges.a.clone(),
                s: ranges.s.clone(),
            };
        }
        'a' => {
            return Ranges {
                x: ranges.x.clone(),
                m: ranges.m.clone(),
                a: constrict_range(&ranges.a, rule),
                s: ranges.s.clone(),
            };
        }
        's' => {
            return Ranges {
                x: ranges.x.clone(),
                m: ranges.m.clone(),
                a: ranges.a.clone(),
                s: constrict_range(&ranges.s, rule),
            };
        }
        c => {
            println!("invalid variable {}", c);
            return ranges.clone();
        }
    }
}
fn is_valid_range(range: &Range) -> bool {
    return range.to >= range.from;
}
fn is_valid_ranges(ranges: &Ranges) -> bool {
    return is_valid_range(&ranges.x)
        && is_valid_range(&ranges.m)
        && is_valid_range(&ranges.a)
        && is_valid_range(&ranges.s);
}
fn get_range_len(range: &Range) -> usize {
    return range.to - range.from + 1;
}
fn calc_accepted_permutations(workflows: &Vec<Workflow>) -> usize {
    let mut explorations: Vec<Exploration> = vec![Exploration {
        target: Target::Target(String::from("in")),
        ranges: Ranges {
            x: Range { from: 1, to: 4000 },
            m: Range { from: 1, to: 4000 },
            a: Range { from: 1, to: 4000 },
            s: Range { from: 1, to: 4000 },
        },
    }];

    let mut accepted: Vec<Ranges> = Vec::new();

    loop {
        match explorations.pop() {
            Some(exploration) => match exploration.target {
                Target::Accept => accepted.push(exploration.ranges),
                Target::Reject => {}
                Target::Target(target) => {
                    match workflows.iter().find(|workflow| workflow.ident == target) {
                        Some(workflow) => {
                            for rule in workflow.rules.iter() {
                                let new_ranges = constrict_ranges(&exploration.ranges, rule);
                                if is_valid_ranges(&new_ranges) {
                                    explorations.push(Exploration {
                                        target: rule.target.clone(),
                                        ranges: new_ranges,
                                    });
                                }
                            }
                        }
                        None => println!("Target could not be found"),
                    }
                }
            },
            None => break,
        }
    }
    // todo: merge ranges

    return accepted
        .iter()
        .map(|ranges| {
            get_range_len(&ranges.x)
                * get_range_len(&ranges.m)
                * get_range_len(&ranges.a)
                * get_range_len(&ranges.s)
        })
        .sum();
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let (workflows, mut parts) = analyze_file(&mut result);

            println!("Permutations: {}", calc_accepted_permutations(&workflows));

            /*for part in parts.iter_mut() {
                run_workflow(&workflows, part);
            }

            let accepted: usize = parts
                .iter()
                .filter(|part| part.current_step == Target::Accept)
                .map(|part| {
                    print!("  Accept: ");
                    for val in part.variables.iter() {
                        print!("{}:{} \t", val.ident, val.value);
                    }
                    println!();
                    part.variables.iter().fold(0, |accu, var| accu + var.value)
                })
                .sum();

            println!("Accepted Sum {accepted}");*/
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
