use llm_msg::Message;
use std::io::{self};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    for input in stdin.lines().map_while(Result::ok) {
        process_input(&input)?;
    }

    Ok(())
}

fn process_input(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let msg: Message = serde_json::from_str(input)?;
    let content = msg.content;

    println!("{content}");
    Ok(())
}
