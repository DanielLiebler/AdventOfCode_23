use std::env;
use std::fs;

struct Entry {
    active: bool,
    start_index: usize,
    end_index: usize,
    number: u32,
}
struct Symbol {
    index: usize,
    adjacient_gears: u32,
    gear_ratio: u32,
    is_gear: bool,
}

fn parse_number(bytes: &[u8], start: usize, end: usize) -> u32 {
    //println!("Trying to parse {}", char::from(slice[0]));
    let mut number: u32 = 0;
    for i in start..end + 1 {
        number = number * 10 + u32::from(bytes[i] - b'0');
    }
    return number;
}

fn add_entry(
    entries: &mut Vec<Vec<Entry>>,
    bytes: &[u8],
    line: usize,
    start: usize,
    end: usize,
    active: bool,
) {
    entries[line].push(Entry {
        active: active,
        start_index: start,
        end_index: end,
        number: parse_number(bytes, start, end),
    });
}

fn add_symbol(symbols: &mut Vec<Vec<Symbol>>, line: usize, index: usize, is_gear: bool) {
    symbols[line].push(Symbol {
        index: index,
        adjacient_gears: 0,
        gear_ratio: 1,
        is_gear: is_gear,
    });
}

fn parse_line(
    entries: &mut Vec<Vec<Entry>>,
    symbols: &mut Vec<Vec<Symbol>>,
    line: usize,
    slice: &[u8],
) {
    let mut start_of_number = 0;
    let mut parsing_number = false;
    let mut is_already_near_symbol = false;
    for (i, &item) in slice.iter().enumerate() {
        if item == b'.' {
            if parsing_number {
                parsing_number = false;
                add_entry(
                    entries,
                    slice,
                    line,
                    start_of_number,
                    i - 1,
                    is_already_near_symbol,
                );
            }
            is_already_near_symbol = false;
        } else if item >= b'0' && item <= b'9' {
            // number
            if parsing_number {
                //
            } else {
                parsing_number = true;
                start_of_number = i;
            }
        } else if item == b'\n' {
            panic!("found NL in Line");
        } else {
            add_symbol(symbols, line, i, item == b'*');
            if parsing_number {
                parsing_number = false;
                add_entry(entries, slice, line, start_of_number, i - 1, true);
            }
            is_already_near_symbol = true;
        }
    }
    if parsing_number {
        add_entry(
            entries,
            slice,
            line,
            start_of_number,
            slice.len() - 1,
            is_already_near_symbol,
        );
    }
}

fn in_range(symbol_index: usize, entry_start: usize, entry_end: usize) -> bool {
    if entry_start == 0 {
        return symbol_index <= entry_end + 1;
    } else {
        return symbol_index >= entry_start - 1 && symbol_index <= entry_end + 1;
    }
}

fn solve_neighbors_across_lines(
    entries: &mut Vec<Vec<Entry>>,
    symbols: &Vec<Vec<Symbol>>,
    lines: usize,
) {
    for i in 0..lines {
        for entry in entries[i].iter_mut() {
            // skip if already marked active
            if entry.active {
                continue;
            }
            if i != 0 {
                for symbol in &symbols[i - 1] {
                    if in_range(symbol.index, entry.start_index, entry.end_index) {
                        entry.active = true;
                    }
                }
            }
            if i != lines - 1 {
                for symbol in &symbols[i + 1] {
                    if in_range(symbol.index, entry.start_index, entry.end_index) {
                        entry.active = true;
                    }
                }
            }
        }
    }
}

fn print_parsed(entries: &mut Vec<Vec<Entry>>, symbols: &mut Vec<Vec<Symbol>>, lines: usize) {
    for i in 0..lines {
        println!("Line {i}:");
        for entry in &entries[i] {
            println!(
                "  E {} {}-{} {}",
                if entry.active { "X" } else { "O" },
                entry.start_index,
                entry.end_index,
                entry.number,
            );
        }
        for symbol in &symbols[i] {
            if symbol.is_gear {
                println!(
                    "  S {} ({}:{})",
                    symbol.index, symbol.adjacient_gears, symbol.gear_ratio
                );
            } else {
                println!("  S {}", symbol.index);
            }
        }
        println!("");
    }
}

fn count_part_numbers(entries: &Vec<Vec<Entry>>, lines: usize) -> u32 {
    let mut sum: u32 = 0;
    for i in 0..lines {
        for entry in &entries[i] {
            if entry.active {
                sum += entry.number;
            }
        }
    }
    return sum;
}

fn calc_gears(entries: &Vec<Vec<Entry>>, symbols: &mut Vec<Vec<Symbol>>, lines: usize) {
    for i in 0..lines {
        for symbol in symbols[i].iter_mut() {
            if symbol.adjacient_gears > 2 || !symbol.is_gear {
                continue;
            }
            if i != 0 {
                for entry in &entries[i - 1] {
                    if in_range(symbol.index, entry.start_index, entry.end_index) {
                        symbol.adjacient_gears += 1;
                        if symbol.adjacient_gears <= 2 {
                            symbol.gear_ratio *= entry.number;
                        }
                    }
                }
            }
            if symbol.adjacient_gears > 2 {
                continue;
            }
            for entry in &entries[i] {
                if in_range(symbol.index, entry.start_index, entry.end_index) {
                    symbol.adjacient_gears += 1;
                    if symbol.adjacient_gears <= 2 {
                        symbol.gear_ratio *= entry.number;
                    }
                }
            }
            if symbol.adjacient_gears > 2 {
                continue;
            }
            if i != lines - 1 {
                for entry in &entries[i + 1] {
                    if in_range(symbol.index, entry.start_index, entry.end_index) {
                        symbol.adjacient_gears += 1;
                        if symbol.adjacient_gears <= 2 {
                            symbol.gear_ratio *= entry.number;
                        }
                    }
                }
            }
        }
    }
}

fn sum_gears(symbols: &Vec<Vec<Symbol>>, lines: usize) -> u32 {
    let mut sum: u32 = 0;
    for i in 0..lines {
        for symbol in &symbols[i] {
            if symbol.adjacient_gears == 2 {
                sum += symbol.gear_ratio;
            }
        }
    }
    return sum;
}

fn main() {
    let mut entries: Vec<Vec<Entry>> = Vec::new();
    let mut symbols: Vec<Vec<Symbol>> = Vec::new();

    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let bytes = contents.as_bytes();
    let mut start = 0;
    let mut cur_line = 0;

    for (i, &item) in bytes.iter().enumerate() {
        if item == b'\n' {
            // make space in vectors
            entries.push(Vec::new());
            symbols.push(Vec::new());

            // parse line
            match bytes.get(start..i - 1) {
                Some(slice) => parse_line(&mut entries, &mut symbols, cur_line, slice),
                None => println!("cannot extract on line {cur_line}: {start}..{}", i),
            }

            // keep track of running variables
            cur_line += 1;
            start = i + 1;
        }
    }

    let total_lines = cur_line;

    solve_neighbors_across_lines(&mut entries, &mut symbols, total_lines);
    calc_gears(&entries, &mut symbols, total_lines);

    print_parsed(&mut entries, &mut symbols, total_lines);

    println!(
        "{total_lines} lines, sum of p.numbers: {}, sum of gear ratios: {}",
        count_part_numbers(&entries, total_lines),
        sum_gears(&symbols, total_lines)
    );
}
