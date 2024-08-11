use core::fmt;
use std::env;
use std::io;
use std::process;

#[derive(Debug, Copy)]
struct RegularExpression {
    alternations: Box<RegularExpression>,
    tokens: Vec<RegexToken>,
}

#[derive(Debug, Clone)]
enum RegexToken {
    Prefix(PrefixMetaCharacter),
    Literal(char),
    Digit,
    Alphanumeric,
    Postfix(PostfixMetaCharacter),
    PositiveGroup(RegularExpression),
    NegativeGroup(RegularExpression),
}

#[derive(Debug, Clone)]
enum PrefixMetaCharacter {
    Backslash,
    Bracket,
    Brace,
    Parenthesis,
}

#[derive(Debug, Clone)]
enum PostfixMetaCharacter {
    Plus(Box<RegularExpression>),
    Star(Box<RegularExpression>),
}

fn match_one(input_char: char, pattern: RegularExpression) -> bool {
    use RegularExpression::*;
    return match pattern {
        Literal(c) => input_char == c,
        Digit => input_char.is_ascii_digit(),
        Alphanumeric => input_char.is_alphanumeric(),
        _ => false,
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let p = compile_pattern(pattern).unwrap();
    println!("{:?}", p);
    let mut c = input_line.chars().peekable();
    if let Some(RegularExpression::Literal('^')) = p.first() {
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

fn compile_one(c: char) -> RegularExpression {
    use RegularExpression::*;
    use PrefixMetaCharacter::*;
    return match c {
        '\\' => Escape(Backslash),
        '[' => Escape(Bracket),
        '{' => Escape(Brace),
        '(' => Escape(Parenthesis), 
        c => Literal(c),
    }
}

fn compile_esc(c: char) -> Result<RegularExpression, RegexError> {
    use RegularExpression::*;
    let token =  match c {
        'd' => Digit,
        'w' => Alphanumeric,
        _ => return Err(RegexError::PatternError),
    };
    Ok(token)
}

fn compile_pattern2(pattern: &str) -> Result<Vec<RegularExpression>, RegexError> {
    use RegularExpression::*;
    use PrefixMetaCharacter::*;
    let mut compiled: Vec<RegularExpression> = Vec::new();
    let mut chars = pattern.chars().peekable();
    while let (Some(cur), next) = (chars.next(), chars.peek()) {
        let re = compile_one(cur);
        let token = match (re, next) {
            (Escape(_), None) => return Err(RegexError::PatternError),
            (Escape(Backslash), Some(second)) => match compile_one(*second) {

                // Plus | Star => Literal(*second),
                _ => return Err(RegexError::PatternError), 
            },
            (re, Some('*')) => Star(Box::new(re)),
            (Escape(Parenthesis), Some('^')) => {
                NegativeGroup(String::new())
            },
            (Escape(Parenthesis), Some(_)) => {
                PositiveGroup(String::new())
            },
        };
        compiled.push(token);
        chars.next();
    }
    Ok(compiled)
}
fn compile_pattern(pattern: &str) -> Result<Vec<RegularExpression>, RegexError> {
    use RegularExpression::*;
    let mut compiled: Vec<RegularExpression> = Vec::new();
    let mut chars = pattern.chars().peekable();
    while let (Some(cur), next) = (chars.next(), chars.peek()) {
        let token = match (cur, next) {
            ('*', Some('*')) => {
                return Err(RegexError::PatternError);
            },
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

fn compile_pattern3(pattern: &str) -> Result<Vec<RegularExpression>, RegexError> {
    use RegularExpression::*;
    let mut compiled: Vec<RegularExpression> = Vec::new();
    let mut chars = pattern.chars().peekable();
    while let (Some(cur), next) = (chars.next(), chars.peek()) {
        let token = match cur {
            '\\' => match next {
                Some('\\') => return Err(RegexError::PatternError),
                Some('d') => {
                    chars.next();
                    Digit
                },
                Some('w') => {
                    chars.next();
                    Alphanumeric
                },
            },
            '[' => match next {
                Some('^') => {
                    let group_members: String = chars.by_ref()
                        .take_while(|x| *x != ']')
                        .collect();
                    NegativeGroup(group_members)
                },
                
            }
            ('*', Some('*')) => {
                return Err(RegexError::PatternError);
            },
            ('[', Some('^')) => {
            },
            ('[', _) => {
                let group_members: String = chars.by_ref()
                     .take_while(|x| *x != ']')
                     .collect();
                PositiveGroup(group_members)
            },
            (c, _) => Literal(c),
            
        };
        compiled.push(token);
    }
    Ok(compiled)
}

fn match_here(mut input: impl Iterator<Item = char>, mut regex: std::iter::Peekable<impl Iterator<Item = RegularExpression>>) -> bool {
    use RegularExpression::*;
    return match (input.next(), regex.peek()) {
        (None, Some(Literal('$'))) => true,
        (_, None) => true, 
        (None, _) => false,
        (Some(input_char), Some(re)) => {
            match re {
                Literal(c) => {
                    if *c == input_char {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                Digit => {
                    if input_char.is_ascii_digit() {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                Alphanumeric => {
                    if input_char.is_alphanumeric() {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                PositiveGroup(members) => {
                    if members.contains(input_char) {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                NegativeGroup(members) => {
                    if !members.contains(input_char) {
                        regex.next();
                        return match_here(input, regex);
                    } else {
                        return false;
                    }
                },
                Plus(plus) => {
                    if !match_one(input_char, *plus.clone()){
                        return false;
                    } else {
                        let mut input_char = input_char;
                        loop {
                            if !match_one(input_char, *plus.clone()){
                                regex.next();
                                return true;
                            }
                            input_char = match input.next() {
                                Some(c) => c,
                                None => {return true;}
                            }
                        }
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
