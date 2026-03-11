use std::{
    collections::HashMap,
    process::{self},
};

type PromptError = Box<dyn std::error::Error>;

struct State {
    path: Option<String>,
    variables: HashMap<String, String>,
    prompt: Option<String>,
}

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

fn parse_args() -> Result<State, PromptError> {
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

    Ok(State {
        path,
        variables,
        prompt: None,
    })
}

fn validate_args(state: State) -> Result<State, PromptError> {
    if state.path.is_none() {
        return Err("missing --template".into());
    }
    if state.variables.is_empty() {
        return Err("at least one --var is required".into());
    }
    Ok(state)
}

fn load_template(state: State) -> Result<State, PromptError> {
    let path = state.path.unwrap();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read template file '{path}': {e}"))?;
    Ok(State {
        path: Some(path),
        variables: state.variables,
        prompt: Some(content),
    })
}

fn substitute_vars(state: State) -> Result<String, PromptError> {
    let content = state.prompt.unwrap();
    let variables = &state.variables;

    let result = content.replace("{{{{", "\x00ESC\x00");

    let mut output = String::new();
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            i += 2;
            let mut var_name = String::new();
            while i < chars.len() {
                if chars[i] == '}' && i + 1 < chars.len() && chars[i + 1] == '}' {
                    i += 2;
                    break;
                }
                var_name.push(chars[i]);
                i += 1;
            }

            if var_name.trim().is_empty() {
                return Err("empty variable name".into());
            }

            let key = var_name.trim();
            let value = variables
                .get(key)
                .ok_or_else(|| format!("undefined variable: {key}"))?;
            output.push_str(value);
        } else {
            output.push(c);
            i += 1;
        }
    }

    let output = output.replace("\x00ESC\x00", "{");
    Ok(output)
}
