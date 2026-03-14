use std::{
    collections::HashMap,
    process::{self},
};

use llm_prompt::substitute;

fn main() {
    let result = parse_args()
        .and_then(validate_args)
        .and_then(load_template)
        .and_then(substitute_vars);

    match result {
        Ok(prompt) => {
            println!("{prompt}");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

#[allow(clippy::type_complexity)]
fn parse_args() -> Result<(Option<String>, HashMap<String, String>), Box<dyn std::error::Error>> {
    let mut path = None;
    let mut variables = HashMap::new();
    let mut args = std::env::args().peekable();

    args.next();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                println!(
                    "Usage: llm_prompt --template <FILE> --var \"KEY=value\" [--var \"KEY2=value2\"]..."
                );
                println!("Options:");
                println!("  --template <FILE>  Path to template file");
                println!(
                    "  --var \"KEY=value\"  Variable substitution (can be specified multiple times)"
                );
                println!("  --help              Display this help information");
                println!("  --version           Display version information");
                process::exit(0);
            }
            "--version" => {
                println!("llm_prompt v0.1.0");
                process::exit(0);
            }
            "--template" => {
                path = Some(args.next().ok_or("missing value for --template")?);
            }
            "--var" => {
                let kv = args.next().ok_or("missing value for --var")?;
                let (key, value) = kv
                    .split_once('=')
                    .ok_or("invalid --var format, expected KEY=value")?;
                if variables.contains_key(key) {
                    return Err(format!("duplicate variable key: {key}").into());
                }
                variables.insert(key.to_string(), value.to_string());
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }

    Ok((path, variables))
}

fn validate_args(
    args: (Option<String>, HashMap<String, String>),
) -> Result<(String, HashMap<String, String>), Box<dyn std::error::Error>> {
    let (path, variables) = args;
    if path.is_none() {
        return Err("missing --template".into());
    }
    if variables.is_empty() {
        return Err("at least one --var is required".into());
    }
    Ok((path.unwrap(), variables))
}

fn load_template(
    args: (String, HashMap<String, String>),
) -> Result<(String, HashMap<String, String>), Box<dyn std::error::Error>> {
    let (path, variables) = args;
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read template file '{path}': {e}"))?;
    Ok((content, variables))
}

fn substitute_vars(
    args: (String, HashMap<String, String>),
) -> Result<String, Box<dyn std::error::Error>> {
    let (content, variables) = args;
    let result: String = substitute(&content, &variables)
        .map_err(|e| -> Box<dyn std::error::Error> { format!("{e}").into() })?;
    Ok(result)
}
