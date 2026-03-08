use llm_msg::Message;
use serde::{Deserialize, Serialize};
use std::io::{self};
use std::time::Duration;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: Option<Message>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lines().map_while(Result::ok).collect();
    let response = process_input(&lines)?;
    println!("{response}");
    Ok(())
}

fn process_input(input: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    // Collect messages from input
    let messages: Vec<Message> = input
        .iter()
        .map(|s| {
            let msg: Message = serde_json::from_str(s).unwrap();
            msg
        })
        .collect();

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let response = chat_with_llm(&client, messages)?;
    let json = serde_json::to_string(&response)?;

    Ok(json)
}

fn chat_with_llm(
    client: &reqwest::blocking::Client,
    messages: Vec<Message>,
) -> Result<Message, Box<dyn std::error::Error>> {
    let request = ChatRequest {
        model: "granite4:3b".to_string(),
        messages,
        stream: false,
    };

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request)
        .send()
        .map_err(|e| {
            format!("Request failed: {e} (hint: check if Ollama is running and has enough memory)",)
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().unwrap_or_default();
        return Err(format!("Ollama error {status}: {error_text}").into());
    }

    let chat_resp: ChatResponse = response.json()?;

    if let Some(msg) = chat_resp.message {
        Ok(msg)
    } else {
        Err("No response from LLM".into())
    }
}
