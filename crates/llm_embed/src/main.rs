use std::{io::Read, process};

use llm_embed::generate;

fn main() {
    let result = parse_args()
        .and_then(validate_args)
        .and_then(generate_embeddings);

    match result {
        Ok(embeddings) => {
            println!("{embeddings}");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

fn parse_args() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut model = None;
    let mut args = std::env::args().peekable();

    args.next();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--model" => {
                model = Some(args.next().ok_or("missing value for --model")?);
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }

    Ok(model)
}

fn validate_args(model: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    model.ok_or_else(|| "missing --model".into())
}

fn generate_embeddings(model: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let input_vec: Vec<String> = serde_json::from_str(&input)?;

    let response = generate(input_vec, &llm_embed::Provider::Ollama { model })?;

    let embeddings = response.embeddings;

    Ok(serde_json::to_string(&embeddings)?)
}
