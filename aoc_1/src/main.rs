use std::env;
use std::fs;

fn reset_spelled_numbers(spellings: &mut Vec<(&'static str, u32, usize)>) {
    for spell in &mut *spellings {
        spell.2 = 0;
    }
}

fn main() {
    // setup spelled numbers
    let mut spellings: Vec<(&'static str, u32, usize)> = vec![
        ("one", 1, 0),
        ("two", 2, 0),
        ("three", 3, 0),
        ("four", 4, 0),
        ("five", 5, 0),
        ("six", 6, 0),
        ("seven", 7, 0),
        ("eight", 8, 0),
        ("nine", 9, 0)
        ];

    // setup input file
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path)
        .expect("Should have been able to read the file");

    let bytes = contents.as_bytes();

    // setup cache values
    let mut first_digit:u32 = 0;
    let mut first_digits:u32 = 0;
    let mut last_digits:u32 = 0;

    let mut num_cache = 0xf;

    // find numbers
    for (i, &item) in bytes.iter().enumerate() {
        if item == b'\n' {
            if num_cache == 0xf {
                panic!("Not enough Numbers in line {i}");
            }
            println!("Line {i}: {},{}", first_digit, u32::from(num_cache));
            last_digits += u32::from(num_cache);
            num_cache = 0xf;
            reset_spelled_numbers(&mut spellings);
        } else if item >= b'0' && item <= b'9' {
            if num_cache == 0xf {
                first_digits += u32::from(item - b'0');
                first_digit = u32::from(item - b'0');
            }
            num_cache = u32::from(item - b'0');
            reset_spelled_numbers(&mut spellings);
        } else {
            for spelling_item in &mut spellings {
                if item == spelling_item.0.as_bytes()[spelling_item.2] {
                    spelling_item.2 += 1;
                    if spelling_item.2 == spelling_item.0.len() {
                        if num_cache == 0xf {
                            first_digit = spelling_item.1;
                            first_digits += spelling_item.1;
                        }
                        num_cache = spelling_item.1;
                        spelling_item.2 = 0;
                    }
                } else {
                    if item == spelling_item.0.as_bytes()[0] {
                        spelling_item.2 = 1;
                    } else {
                        spelling_item.2 = 0;
                    }
                }
            }
        }
    }
    println!("Calculated Result as {}", first_digits * 10 + last_digits);
}
