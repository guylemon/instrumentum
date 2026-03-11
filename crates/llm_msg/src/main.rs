use llm_msg::Message;
use std::env;
use std::io::{self};

fn main() {
    let (role, msg) = parse_args();

    if msg.is_empty() {
        // Fall back to stdin for message
        let stdin = io::stdin();
        for input in stdin.lines().map_while(Result::ok) {
            process_input(&input, &role);
        }
    } else {
        process_input(&msg, &role);
    }
}

fn parse_args() -> (String, String) {
    let arguments: Vec<String> = env::args().collect();
    let mut role = "user".to_string();
    let mut msg = String::new();

    let mut args = arguments[1..].iter();

    // [--role user|system] [input]
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--role" => {
                if let Some(r) = args.next() {
                    role.clone_from(r);
                }
            }
            input => {
                msg = input.to_string();
                break;
            }
        }
    }

    (role, msg)
}

fn process_input(input: &str, role: &str) {
    let msg = Message::new(role, input);
    let json = serde_json::to_string(&msg).unwrap();

    println!("{json}");
}
