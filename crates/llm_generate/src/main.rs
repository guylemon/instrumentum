use llm_generate::{Message, Provider, generate};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (provider, tools_enabled, context_file) = parse_args();

    if let Some(path) = context_file {
        let lines: Vec<String> = std::fs::read_to_string(&path)?
            .lines()
            .map(String::from)
            .collect();
        let response = process_input(&lines, &provider, tools_enabled)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(file, "{response}")?;
    } else {
        let stdin = io::stdin();
        let lines: Vec<String> = stdin.lines().map_while(Result::ok).collect();
        let response = process_input(&lines, &provider, tools_enabled)?;
        println!("{response}");
    }
    Ok(())
}

fn parse_args() -> (Provider, bool, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    let mut model: Option<String> = None;
    let mut tools_enabled = false;
    let mut context_file = None;

    for i in 0..args.len().saturating_sub(1) {
        if args[i] == "--model" {
            model = Some(args[i + 1].clone());
        }
        if args[i] == "--provider" && args[i + 1] == "xai" {
            let api_key = std::env::var("XAI_API_KEY")
                .expect("XAI_API_KEY environment variable must be set when using --provider xai");
            let model = model
                .or_else(|| std::env::var("XAI_MODEL").ok())
                .unwrap_or_else(|| "grok-4-1-fast-reasoning".to_string());
            return (
                Provider::Xai { api_key, model },
                tools_enabled,
                context_file,
            );
        }
        if args[i] == "--tools" {
            tools_enabled = true;
        }
        if args[i] == "--context" {
            context_file = Some(args[i + 1].clone());
        }
    }

    let model = model
        .or_else(|| std::env::var("OLLAMA_MODEL").ok())
        .unwrap_or_else(|| "qwen3:8b".to_string());

    (Provider::Ollama { model }, tools_enabled, context_file)
}

fn process_input(
    input: &[String],
    provider: &Provider,
    tools_enabled: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let messages: Vec<Message> = input
        .iter()
        .map(|s| {
            let msg: Message = serde_json::from_str(s).unwrap();
            msg
        })
        .collect();

    let final_response = generate(messages, tools_enabled, provider)?;

    let json = serde_json::to_string(&final_response)?;

    Ok(json)
}
