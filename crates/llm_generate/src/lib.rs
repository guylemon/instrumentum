use llm_provider::Config;
use serde::{Deserialize};
use std::io::Write;
use std::time::Duration;

pub use llm_msg::Message;
use llm_provider::ChatRequest;
use llm_provider::Provider;
use llm_provider::Tool;

#[derive(Deserialize)]
struct ChatResponse {
    message: Option<Message>,
}

#[derive(Deserialize)]
struct XaiChatResponse {
    choices: Vec<XaiChoice>,
}

#[derive(Deserialize)]
struct XaiChoice {
    message: Message,
}

/// Generate a response from an LLM using the provided messages and tools.
/// # Errors
/// Returns an error if the LLM request fails or returns an invalid response.
pub fn generate(
    chat_request: &ChatRequest,
    provider: &Provider,
) -> Result<Message, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let mut all_messages = chat_request.messages.clone();
    let final_response = loop {
        let response = chat_with_llm(&client, chat_request, provider)?;
        all_messages.push(response.clone());

        if let Some(ref tool_calls) = response.tool_calls {
            if tool_calls.is_empty() {
                break response;
            }

            for tc in tool_calls {
                let id = tc.id.clone();
                let name = tc.function.name.clone();
                let args = tc.function.arguments.clone();
                eprintln!("...calling tool: {name}({args:?})");

                let result = execute_tool(&name, &args)?;
                eprintln!("...result: {result}");
                all_messages.push(Message {
                    role: llm_msg::Role::Tool,
                    content: result,
                    tool_call_id: id,
                    tool_name: Some(name),
                    tool_calls: None,
                });
            }
        } else {
            break response;
        }
    };

    Ok(final_response)
}

// TODO refactor chat_with_xai to use ChatRequest
fn chat_with_llm(
    client: &reqwest::blocking::Client,
    chat_request: &ChatRequest,
    provider: &Provider,
) -> Result<Message, Box<dyn std::error::Error>> {
    match provider {
        Provider::Ollama(config) => chat_with_ollama(client, chat_request, config),
        Provider::Xai { api_key, model } => chat_with_xai(
            client,
            &chat_request.messages,
            chat_request.tools.as_ref(),
            api_key,
            model,
        ),
    }
}

fn chat_with_ollama(
    client: &reqwest::blocking::Client,
    chat_request: &ChatRequest,
    config: &Config,
) -> Result<Message, Box<dyn std::error::Error>> {
    let url = format!("{}/chat", config.base_url);
    let response = client
        .post(url)
        .json(&chat_request)
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

fn chat_with_xai(
    client: &reqwest::blocking::Client,
    messages: &[Message],
    tools: Option<&Vec<Tool>>,
    api_key: &str,
    model: &str,
) -> Result<Message, Box<dyn std::error::Error>> {
    let default_tools = vec![];
    let tools = tools.unwrap_or(&default_tools);
    let request = ChatRequest::builder(model)
        .messages(messages.to_vec())
        .tools(tools.clone())
        .options(llm_provider::Options::recommended())
        .build()?;

    let response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().unwrap_or_default();
        return Err(format!("XAI error {status}: {error_text}").into());
    }

    let chat_resp: XaiChatResponse = response.json()?;

    if let Some(choice) = chat_resp.choices.into_iter().next() {
        Ok(choice.message)
    } else {
        Err("No response from LLM".into())
    }
}

// TODO move tool execution code to tools. Could just be tool.execute(args) or something.
fn execute_tool(
    name: &str,
    args: &serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
    let args = match args {
        serde_json::Value::Object(_) => args.clone(),
        serde_json::Value::String(s) => {
            serde_json::from_str(s).map_err(|e| format!("Failed to parse tool arguments: {e}"))?
        }
        _ => return Err("Invalid arguments format".into()),
    };

    match name {
        "websearch" => {
            let queries: Vec<String> = args["queries"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            eprintln!("...searching web for {queries:?}");

            let input = queries.join("\n");
            let mut child = std::process::Command::new("websearch")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;

            child.stdin.as_mut().unwrap().write_all(input.as_bytes())?;

            let output = child.wait_with_output()?;

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        "webfetch" => {
            let urls: Vec<String> = args["urls"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            eprintln!("...fetching URLs: {urls:?}");

            let input = urls.join("\n");
            let mut child = std::process::Command::new("webfetch")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;

            child.stdin.as_mut().unwrap().write_all(input.as_bytes())?;

            let output = child.wait_with_output()?;

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        _ => Err(format!("Unknown tool: {name}").into()),
    }
}
