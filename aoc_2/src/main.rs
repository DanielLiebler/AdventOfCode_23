use std::env;
use std::fs;

struct Game {
    game_id: u32,
    max_red: u32,
    max_green: u32,
    max_blue: u32,
}
enum Color {
    Red,
    Green,
    Blue,
}

fn parse_color(slice: &[u8]) -> Color {
    if slice[0] == b'r' {
        return Color::Red;
    } else if slice[0] == b'g' {
        return Color::Green;
    } else if slice[0] == b'b' {
        return Color::Blue;
    }
    println!("Not a color, returning red");
    return Color::Red;
}

fn parse_number(slice: &[u8]) -> u32 {
    //println!("Trying to parse {}", char::from(slice[0]));
    let mut number: u32 = 0;
    for (_i, &item) in slice.iter().enumerate() {
        number = number * 10 + u32::from(item - b'0');
    }
    return number;
}

fn parse_game(games: &mut Vec<Game>, slice: &[u8]) {
    let mut game = Game {
        game_id: 0,
        max_red: 0,
        max_green: 0,
        max_blue: 0,
    };
    let mut start_num: usize = 0;
    let mut num: u32 = 0;
    let mut start_color: usize = 0;
    let mut skip = false;

    for i in 5..slice.len() {
        if skip == true {
            skip = false;
            continue;
        }
        if slice[i] == b':' {
            game.game_id = parse_number(&slice[5..i]);
            skip = true; // skip space
            start_num = i + 2;
        } else if slice[i] == b' ' {
            num = parse_number(&slice[start_num..i]);
            start_color = i + 1;
        } else if slice[i] == b',' || slice[i] == b';' {
            match parse_color(&slice[start_color..i]) {
                Color::Red => {
                    if num > game.max_red {
                        game.max_red = num;
                    }
                }
                Color::Green => {
                    if num > game.max_green {
                        game.max_green = num;
                    }
                }
                Color::Blue => {
                    if num > game.max_blue {
                        game.max_blue = num;
                    }
                }
            }
            start_num = i + 2;
            skip = true; // skip space
        }
    }
    match parse_color(&slice[start_color..slice.len()]) {
        Color::Red => {
            if num > game.max_red {
                game.max_red = num;
            }
        }
        Color::Green => {
            if num > game.max_green {
                game.max_green = num;
            }
        }
        Color::Blue => {
            if num > game.max_blue {
                game.max_blue = num;
            }
        }
    }
    games.push(game);
}

fn main() {
    let mut games: Vec<Game> = Vec::new();

    // setup input file
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let bytes = contents.as_bytes();
    let mut start = 0;
    for (i, &item) in bytes.iter().enumerate() {
        if item == b'\n' {
            match bytes.get(start..i - 1) {
                Some(slice) => parse_game(&mut games, slice),
                None => println!("cannot extract {start}..{}", i - 1),
            }
            start = i + 1;
        }
    }
    println!("Found {} Games", games.len());

    let mut sum: u32 = 0;
    let mut powers: u32 = 0;
    for game in games {
        if game.max_red <= 12 && game.max_green <= 13 && game.max_blue <= 14 {
            sum += game.game_id;
        }

        let power: u32 = game.max_red * game.max_green * game.max_blue;
        powers += power;

        println!(
            "ID{}: R{}, G{},B{}, Pow{}",
            game.game_id, game.max_red, game.max_green, game.max_blue, power
        );
    }
    println!("Sum is {sum}");
    println!("Power is {powers}");
}
