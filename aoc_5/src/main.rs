use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

struct Mapping {
    start_source: usize,
    start_dest: usize,
    len: usize,
}
struct Map {
    from: String,
    to: String,
    mappings: Vec<Mapping>,
}
impl Map {
    pub fn new() -> Self {
        Self {
            from: String::new(),
            to: String::new(),
            mappings: Vec::new(),
        }
    }
}

fn analyze_number_pair(pair: Pair<'_, Rule>) -> Result<(usize, usize), &str> {
    let mut num: Option<usize> = None;
    let mut count: Option<usize> = None;
    for n in pair.into_inner() {
        match n.as_rule() {
            Rule::number => match n.as_str().parse::<usize>() {
                Ok(i) => num = Some(i),
                Err(e) => println!("Error parsing number {e}"),
            },
            Rule::count => match n.as_str().parse::<usize>() {
                Ok(i) => count = Some(i),
                Err(e) => println!("Error parsing number {e}"),
            },
            Rule::EOI => {
                println!("      EOI {}", n.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(NumberPair) {}", n.as_str());
            }
        }
    }

    match (num, count) {
        (Some(i), Some(j)) => return Ok((i, j)),
        _ => return Err("Could not parse pair"),
    }
}

fn analyze_number_list(entry: Pair<'_, Rule>) -> Vec<(usize, usize)> {
    let mut seeds: Vec<(usize, usize)> = Vec::new();

    for number in entry.into_inner() {
        match number.as_rule() {
            Rule::number_pair => match analyze_number_pair(number) {
                Ok((num, count)) => {
                    if count > 0 {
                        seeds.push((num, count));
                    }
                }
                Err(_e) => {}
            },
            Rule::EOI => {
                println!("    EOI {}", number.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(NumberList) {}", number.as_str());
            }
        }
    }

    return seeds;
}

fn analyze_seeds(parsed: Pair<'_, Rule>) -> Option<Vec<(usize, usize)>> {
    let mut seeds: Option<Vec<(usize, usize)>> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::number_list => seeds = Some(analyze_number_list(entry)),
            Rule::EOI => {
                println!("  EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(Seeds) {}", entry.as_str());
            }
        }
    }

    return seeds;
}

fn analyze_map_header(header: Pair<'_, Rule>) -> Result<(String, String), &'static str> {
    let mut from: Option<String> = None;
    let mut to: Option<String> = None;
    for entry in header.into_inner() {
        match entry.as_rule() {
            Rule::from => {
                from = Some(entry.as_str().to_string());
            }
            Rule::to => {
                to = Some(entry.as_str().to_string());
            }
            Rule::EOI => {
                println!("    EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(map header) {}", entry.as_str());
            }
        }
    }
    match (from, to) {
        (Some(f), Some(t)) => return Ok((f, t)),
        _ => return Err("Entry could not be parsed completely"),
    }
}
fn analyze_map_entry(entry: Pair<'_, Rule>) -> Result<Mapping, &'static str> {
    let mut start_source: Option<usize> = None;
    let mut start_dest: Option<usize> = None;
    let mut len: Option<usize> = None;
    for property in entry.into_inner() {
        match property.as_rule() {
            Rule::start_src => match property.as_str().parse::<usize>() {
                Ok(i) => start_source = Some(i),
                Err(_e) => {
                    return Err("Error parsing source start");
                }
            },
            Rule::start_dest => match property.as_str().parse::<usize>() {
                Ok(i) => start_dest = Some(i),
                Err(_e) => {
                    return Err("Error parsing destination start");
                }
            },
            Rule::length => match property.as_str().parse::<usize>() {
                Ok(i) => len = Some(i),
                Err(_e) => {
                    return Err("Error parsing length");
                }
            },
            Rule::EOI => {
                println!("    EOI {}", property.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(map header) {}", property.as_str());
            }
        }
    }
    match (start_source, start_dest, len) {
        (Some(start_source), Some(start_dest), Some(len)) => {
            return Ok(Mapping {
                start_source: start_source,
                start_dest: start_dest,
                len: len,
            });
        }
        _ => return Err("Entry could not be parsed completely"),
    }
}

fn analyze_map(entry: Pair<'_, Rule>) -> Result<Map, &str> {
    let mut map = Map::new();
    for entry in entry.into_inner() {
        match entry.as_rule() {
            Rule::map_header => match analyze_map_header(entry) {
                Ok((f, t)) => {
                    map.from = f;
                    map.to = t;
                }
                Err(e) => {
                    return Err(e);
                }
            },
            Rule::map_entry => match analyze_map_entry(entry) {
                Ok(mapping) => map.mappings.push(mapping),
                Err(e) => println!("Error while parsing mapping {e}"),
            },
            Rule::EOI => {
                println!("  EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(map) {}", entry.as_str());
            }
        }
    }
    return Ok(map);
}

fn analyze_file(
    parsed: &mut Pairs<'_, Rule>,
) -> Result<(Vec<(usize, usize)>, Vec<Map>), &'static str> {
    let unwrapped = parsed.next().unwrap();
    let mut seeds: Option<Vec<(usize, usize)>> = None;
    let mut maps: Vec<Map> = Vec::new();

    for line in unwrapped.into_inner() {
        match line.as_rule() {
            Rule::seeds => {
                seeds = analyze_seeds(line);
            }
            Rule::map => match analyze_map(line) {
                Ok(m) => maps.push(m),
                Err(e) => println!("Error while parsing map {e}"),
            },
            Rule::EOI => {
                println!("EOI {}", line.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", line.as_str());
            }
        }
    }
    match seeds {
        Some(s) => return Ok((s, maps)),
        None => Err("No seeds parsed"),
    }
}

fn get_next_map<'a>(maps: &'a Vec<Map>, precedent: Option<&Map>) -> Option<&'a Map> {
    match precedent {
        Some(precedent) => return maps.iter().find(|m| m.from == precedent.to),
        None => return maps.iter().find(|m| m.from == "seed".to_string()),
    }
}

fn print_mapping(mapped_to: &Vec<(usize, usize)>, precedent: Option<&Map>) {
    print!(
        "{}: ",
        precedent.map_or("seed".to_string(), |p| p.from.clone())
    );

    for (num, cnt) in mapped_to {
        print!("{},{} ", num, cnt);
    }
    println!("");
}

fn intersects(start_a: usize, len_a: usize, start_b: usize, len_b: usize) -> bool {
    let start_a_intersects = start_a >= start_b && start_a < start_b + len_b;
    let start_b_intersects = start_b >= start_a && start_b < start_a + len_a;
    return start_a_intersects || start_b_intersects;
}

fn print_maps(maps: &Vec<Map>) {
    for map in maps {
        println!("Map: {}->{}", map.from, map.to);
        for mapping in &map.mappings {
            println!(
                "{}-{} -> {}-{}, ({})",
                mapping.start_source,
                mapping.start_source + mapping.len,
                mapping.start_dest,
                mapping.start_dest + mapping.len,
                mapping.len
            );
        }
    }
}

fn solve_seeding(seeds: Vec<(usize, usize)>, maps: Vec<Map>) -> usize {
    //print_maps(&maps);

    let mut mapped_to = seeds;
    let mut precedent: Option<&Map> = None;

    loop {
        //print_mapping(&mapped_to, precedent);
        match get_next_map(&maps, precedent) {
            Some(m) => {
                precedent = Some(m);
                mapped_to = mapped_to
                    .iter()
                    .flat_map(|&(num, cnt)| {
                        let mut mapped = Vec::new();
                        let mut remaining = vec![(num, cnt)];
                        m.mappings.iter().for_each(|m| {
                            remaining = remaining
                                .iter()
                                .flat_map(|&(start, len)| {
                                    if m.start_source <= start
                                        && m.start_source + m.len >= start + len
                                    {
                                        // is completely moved
                                        mapped.push((start + m.start_dest - m.start_source, len));
                                        return vec![];
                                    } else if m.start_source <= start
                                        && m.start_source + m.len > start
                                    {
                                        // only starting part is moved
                                        let moved_len = m.start_source + m.len - start;
                                        mapped.push((
                                            start + m.start_dest - m.start_source,
                                            moved_len,
                                        ));
                                        return vec![(start + moved_len, len - moved_len)];
                                    } else if m.start_source < start + len
                                        && m.start_source + m.len >= start + len
                                    {
                                        // only ending part is moved
                                        let moved_len = start + len - m.start_source;
                                        mapped.push((m.start_dest, moved_len));
                                        return vec![(start, len - moved_len)];
                                    } else if m.start_source > start
                                        && m.start_source + m.len < start + len
                                    {
                                        // only part in the middle
                                        let first_remaining_len = m.start_source - start;
                                        let moved_len = m.len;
                                        let second_remaining_len =
                                            start + len - m.start_source - m.len;
                                        mapped.push((m.start_dest, moved_len));
                                        return vec![
                                            (start, first_remaining_len),
                                            (
                                                start + first_remaining_len + moved_len,
                                                second_remaining_len,
                                            ),
                                        ];
                                    } else {
                                        return vec![(start, len)];
                                    }
                                })
                                .collect::<Vec<(usize, usize)>>();
                        });
                        mapped.append(&mut remaining);
                        return mapped;
                    })
                    .collect::<Vec<(usize, usize)>>();
            }
            None => {
                //print_mapping(&mapped_to, precedent);
                break;
            }
        }
    }

    mapped_to.sort();
    return mapped_to[0].0;
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => match analyze_file(&mut result) {
            Ok((seeds, maps)) => {
                let location = solve_seeding(seeds, maps);
                println!("closest location is {location}");
            }
            Err(e) => println!("Error parsing file: {e}"),
        },
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
