use core::fmt;
use std::env;
use std::io;
use std::iter::Cloned;
use std::process;
use std::slice::Iter;

#[derive(Debug, Clone)]
enum RegularExpressionElement {
    Literal(char),
    Digit,
    Alphanumeric,
    Star(Box<RegularExpressionElement>),
    PositiveGroup(String),
    NegativeGroup(String),
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let p = compile_pattern(pattern).unwrap();
    println!("{:?}", p);
    let mut c = input_line.chars().peekable();
    if let Some(RegularExpressionElement::Literal('^')) = p.first() {
        let mut p_iter = p.iter().map(Clone::clone).peekable();
        p_iter.next();
        return match_here(c, p_iter)
    }
    loop {
        println!("{:?}", c.clone());
        let p_iter = p.iter().map(Clone::clone).peekable();
        if match_here(c.clone(), p_iter) {
            return true;
        }
        if c.peek().is_some() {
            c.next();
        } else {
            return false;
        }
    }
}

#[derive(Debug)]
enum RegexError {
    PatternError,
}

impl std::error::Error for RegexError {}
impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexError::PatternError => write!(f, "Pattern Error"),
        }
    }
}

fn compile_pattern(pattern: &str) -> Result<Vec<RegularExpressionElement>, RegexError> {
    use RegularExpressionElement::*;
    let mut compiled: Vec<RegularExpressionElement> = Vec::new();
    let mut chars = pattern.chars().peekable();
    while let (Some(cur), next) = (chars.next(), chars.peek()) {
        let token = match (cur, next) {
            ('*', Some('*')) => {
                return Err(RegexError::PatternError);
            }
            ('[', Some('^')) => {
                let group_members: String = chars.by_ref()
                     .take_while(|x| *x != ']')
                     .collect();
                NegativeGroup(group_members)
            },
            ('[', _) => {
                let group_members: String = chars.by_ref()
                     .take_while(|x| *x != ']')
                     .collect();
                PositiveGroup(group_members)
            },
            ('\\', Some('d')) => {
                chars.next();
                Digit
            },
            ('\\', Some('w')) => {
                chars.next();
                Alphanumeric
            },
            (c, _) => Literal(c),
            
        };
        compiled.push(token);
    }
    Ok(compiled)
}

fn match_here(mut input: impl Iterator<Item = char>, mut regex: std::iter::Peekable<impl Iterator<Item = RegularExpressionElement>>) -> bool {
    use RegularExpressionElement::*;
    return match (input.next(), regex.peek()) {
        (None, Some(Literal('$'))) => true,
        (_, None) => true, 
        (None, _) => false,
        (Some(one_input), Some(re)) => {
            match re {
                Literal(c) => {
                    if *c == one_input {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                Digit => {
                    if one_input.is_ascii_digit() {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                Alphanumeric => {
                    if one_input.is_alphanumeric() {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                PositiveGroup(members) => {
                    if members.contains(one_input) {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                NegativeGroup(members) => {
                    if !members.contains(one_input) {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                }
                _ => false

            }
        }
    };
}
fn match_star() {

}
    // if let y = pattern.chars().nth(0) {
    //     // everything
    // }
    // let x = pattern[1..];
    // match pattern {
    //     "\\d" => input_line.contains(|x: char| x.is_ascii_digit()),
    //     "\\w" => input_line.contains(|x: char| x.is_alphanumeric()),
    //     _ if pattern.chars().count() == 1 => input_line.contains(pattern),
    //     m if match_negative_group(pattern) => {
    //         let chars: String = m[2..].into();
    //         for c in chars.chars() {
    //             if input_line.contains(c) {
    //                 return false;
    //             }
    //         }
    //         return true;
    //     },
    //     m if match_positive_group(pattern) => {
    //         let chars: String = m[1..].into();
    //         for c in chars.chars() {
    //             if input_line.contains(c) {
    //                 return true;
    //             }
    //         }
    //         return false;
    //     },
    //     _ => panic!("unhandled pattern: {}", pattern)
    // }

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // Uncomment this block to pass the first stage
    if match_pattern(&input_line, &pattern) {
        println!("matched!");
        process::exit(0)
    } else {
        process::exit(1)
    }
}
