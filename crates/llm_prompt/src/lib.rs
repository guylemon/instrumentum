use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PromptError {
    EmptyVariableName,
    UndefinedVariable(String),
}

impl std::fmt::Display for PromptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptError::EmptyVariableName => write!(f, "empty variable name"),
            PromptError::UndefinedVariable(name) => write!(f, "undefined variable: {name}"),
        }
    }
}

impl std::error::Error for PromptError {}

#[allow(clippy::implicit_hasher)]
#[allow(clippy::missing_errors_doc)]
pub fn substitute(
    template: &str,
    variables: &HashMap<String, String>,
) -> Result<String, PromptError> {
    let result = template.replace("{{{{", "\x00ESC\x00");

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
                return Err(PromptError::EmptyVariableName);
            }

            let key = var_name.trim();
            let value = variables
                .get(key)
                .ok_or_else(|| PromptError::UndefinedVariable(key.to_string()))?;
            output.push_str(value);
        } else {
            output.push(c);
            i += 1;
        }
    }

    let output = output.replace("\x00ESC\x00", "{");
    Ok(output)
}
