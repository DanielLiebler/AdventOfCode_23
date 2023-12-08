use std::cmp::Ordering;
use std::env;
use std::fs;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    J,
    Q,
    K,
    A,
    None,
}
struct Hand {
    cards: Vec<Card>,
    bid: u32,
}

impl Card {
    pub fn from_str(str: &str) -> Card {
        match str.chars().next().expect("could not parse Card") {
            '2' => Card::Two,
            '3' => Card::Three,
            '4' => Card::Four,
            '5' => Card::Five,
            '6' => Card::Six,
            '7' => Card::Seven,
            '8' => Card::Eight,
            '9' => Card::Nine,
            'T' => Card::Ten,
            'J' => Card::J,
            'Q' => Card::Q,
            'K' => Card::K,
            'A' => Card::A,
            _ => Card::None,
        }
    }
    fn as_value(&self) -> i32 {
        match self {
            Card::None => 0,
            Card::Two => 2,
            Card::Three => 3,
            Card::Four => 4,
            Card::Five => 5,
            Card::Six => 6,
            Card::Seven => 7,
            Card::Eight => 8,
            Card::Nine => 9,
            Card::Ten => 10,
            Card::J => 1,
            Card::Q => 12,
            Card::K => 13,
            Card::A => 14,
        }
    }
}

impl PartialEq for Card {
    fn eq(&self, other: &Card) -> bool {
        self.as_value() == other.as_value()
    }
}
impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Card) -> Option<Ordering> {
        if *self == Card::None {
            return Some(Ordering::Less);
        } else if *other == Card::None {
            return Some(Ordering::Greater);
        }
        let self_value = self.as_value();
        let other_value = other.as_value();
        Some(self_value.cmp(&other_value))
    }
}
struct CardCount {
    card: Card,
    count: u32,
}

impl Hand {
    fn get_hand_value(&self) -> u32 {
        let mut accu: Vec<CardCount> = vec![
            CardCount {
                card: Card::Two,
                count: 0,
            },
            CardCount {
                card: Card::Three,
                count: 0,
            },
            CardCount {
                card: Card::Four,
                count: 0,
            },
            CardCount {
                card: Card::Five,
                count: 0,
            },
            CardCount {
                card: Card::Six,
                count: 0,
            },
            CardCount {
                card: Card::Seven,
                count: 0,
            },
            CardCount {
                card: Card::Eight,
                count: 0,
            },
            CardCount {
                card: Card::Nine,
                count: 0,
            },
            CardCount {
                card: Card::Ten,
                count: 0,
            },
            CardCount {
                card: Card::J,
                count: 0,
            },
            CardCount {
                card: Card::Q,
                count: 0,
            },
            CardCount {
                card: Card::K,
                count: 0,
            },
            CardCount {
                card: Card::A,
                count: 0,
            },
            CardCount {
                card: Card::None,
                count: 0,
            },
        ];
        for c in &self.cards {
            if *c == Card::J {
                for cc in accu.iter_mut() {
                    cc.count += 1;
                }
            } else {
                match accu.iter_mut().find(|cc| cc.card.eq(c)) {
                    Some(cc) => cc.count += 1,
                    None => println!("Err: Lost cardcount!"),
                };
            }
        }
        accu.sort_by(|a, b| {
            return b.count.cmp(&a.count);
        });
        let jokers = match accu.iter_mut().find(|cc| cc.card.eq(&Card::J)) {
            Some(cc) => cc.count,
            None => 0,
        };
        match accu.get(0).expect("Err: Lost cardcount later!").count {
            5 => return 6, // five of a kind
            4 => return 5, // four of a kind
            i if i == 3 => match accu.get(1).expect("Err: Lost cardcount later!").count {
                j if j >= 1 => {
                    if i + j - jokers >= 5 {
                        return 4;
                    } else {
                        return 3;
                    }
                } // full house
                _ => println!("unexpected count!"),
            },
            i if i == 2 => match accu.get(1).expect("Err: Lost cardcount later!").count {
                j if j >= 1 => {
                    if i + j - jokers >= 4 {
                        return 2;
                    } else {
                        return 1;
                    }
                } // full house
                _ => println!("unexpected count!"),
            },
            1 => return 0, // high card
            _ => println!("unexpected count!3"),
        }
        return 0;
    }
}

impl PartialEq for Hand {
    fn eq(&self, other: &Hand) -> bool {
        for (s, o) in self.cards.iter().zip(other.cards.iter()) {
            if *s != *o {
                return false;
            }
        }
        return true;
    }
}
impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Hand) -> Option<Ordering> {
        // compare hand types (hand values)
        match self.get_hand_value().cmp(&other.get_hand_value()) {
            Ordering::Less => return Some(Ordering::Less),
            Ordering::Greater => return Some(Ordering::Greater),
            Ordering::Equal => {}
        }

        // compare equal hand types (card values)
        for (s, o) in self.cards.iter().zip(other.cards.iter()) {
            match s.partial_cmp(o) {
                Some(Ordering::Equal) => {}
                Some(o) => return Some(o),
                None => return None,
            }
        }
        return Some(Ordering::Equal);
    }
}
impl Eq for Hand {}
impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.partial_cmp(other).expect("Could not order Hand");
    }
}

fn analyze_line(parsed: Pair<'_, Rule>) -> Result<Hand, &'static str> {
    let mut cards: Vec<Card> = Vec::new();
    let mut bid: Option<u32> = None;
    for entry in parsed.into_inner() {
        match entry.as_rule() {
            Rule::card => {
                cards.push(Card::from_str(entry.as_str()));
            }
            Rule::number => match entry.as_str().parse::<u32>() {
                Ok(i) => bid = Some(i),
                Err(e) => println!("Error parsing number {e}"),
            },
            Rule::EOI => {
                println!("  EOI {}", entry.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(entry) {}", entry.as_str());
            }
        }
    }
    return match bid {
        Some(b) => Ok(Hand {
            cards: cards,
            bid: b,
        }),
        None => Err("Could not parse hand, missing bid"),
    };
}

fn analyze_file(parsed: &mut Pairs<'_, Rule>) -> Vec<Hand> {
    let mut hands: Vec<Hand> = Vec::new();
    let unwrapped = parsed.next().unwrap();
    for line in unwrapped.into_inner() {
        match line.as_rule() {
            Rule::line => match analyze_line(line) {
                Ok(hand) => hands.push(hand),
                Err(e) => print!("Error parsing line: {e}"),
            },
            Rule::EOI => {
                println!("EOI {}", line.as_str());
            }
            _ => {
                println!("UNEXPECTED PARSE(File) {}", line.as_str());
            }
        }
    }
    return hands;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let parse_result = MyParser::parse(Rule::file, &contents);

    match parse_result {
        Ok(mut result) => {
            let mut hands = analyze_file(&mut result);
            hands.sort();
            println!("Sorted:");
            for h in &hands {
                print!("C({}, {}):", h.bid, h.get_hand_value());
                for c in &h.cards {
                    print!("{},", c.as_value());
                }
                println!("");
            }
            let sum = hands.iter().enumerate().fold(0, |accu, (i, h)| {
                accu + u32::try_from(i + 1).unwrap() * h.bid
            });
            println!("Sum is {sum}");
        }
        Err(result) => {
            println!("ERR:  Could not parse file: {result}");
        }
    }
}
