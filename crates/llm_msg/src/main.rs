use llm_msg::Message;
use std::env;
use std::io::{self};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Use first positional arg as input
        let input = &args[1];
        process_input(input);
    } else {
        // Fall back to stdin
        let stdin = io::stdin();
        for input in stdin.lines().map_while(Result::ok) {
            process_input(&input);
        }
    }
}

fn process_input(input: &str) {
    let role = "user";
    let content = input;

    let msg = Message::new(role, content);

    let json = serde_json::to_string(&msg).unwrap();

    println!("{json}");
}
