use llm_msg::Message;
use std::env;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let (role, msg) = parse_args();

    if msg.is_empty() {
        // Fall back to stdin for message
        let mut full_input = String::new();
        io::stdin().read_to_string(&mut full_input).ok();
        if full_input.is_empty() {
            Err("LLM message must not be empty".into())
        } else {
            process_input(&full_input, &role)
        }
    } else {
        process_input(&msg, &role)
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

fn process_input(input: &str, role: &str) -> Result<(), Box<dyn std::error::Error>> {
    let role = role.parse()?;
    let msg = Message::new(role, input);
    let json = serde_json::to_string(&msg).unwrap();

    println!("{json}");
    Ok(())
}
