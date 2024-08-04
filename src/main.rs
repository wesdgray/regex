use std::env;
use std::io;
use std::process;

fn match_positive_group(pattern: &str) -> bool {
    pattern.chars().nth(0).unwrap_or(' ') == '[' && pattern.chars().last().unwrap_or(' ') == ']'
}
fn match_negative_group(pattern: &str) -> bool {
    match_positive_group(pattern) && pattern.chars().nth(1).unwrap_or(' ') == '^'
}
fn match_pattern(input_line: &str, pattern: &str) -> bool {
    match pattern {
        "\\d" => input_line.contains(|x: char| x.is_ascii_digit()),
        "\\w" => input_line.contains(|x: char| x.is_alphanumeric()),
        _ if pattern.chars().count() == 1 => input_line.contains(pattern),
        m if match_negative_group(pattern) => {
            let chars: String = m[2..].into();
            for c in chars.chars() {
                if input_line.contains(c) {
                    return false;
                }
            }
            return true;
        },
        m if match_positive_group(pattern) => {
            let chars: String = m[1..].into();
            for c in chars.chars() {
                if input_line.contains(c) {
                    return true;
                }
            }
            return false;
        },
        _ => panic!("unhandled pattern: {}", pattern)
    }
}

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
        process::exit(0)
    } else {
        process::exit(1)
    }
}
