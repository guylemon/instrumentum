use llm_msg::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self, Write};
use std::time::Duration;

#[derive(Clone)]
enum Provider {
    Ollama,
    Xai { api_key: String, model: String },
}

#[derive(Serialize)]
struct ChatRequestOptions {
    temperature: u8,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    options: ChatRequestOptions,
    stream: bool,
    tools: Option<Vec<Tool>>,
}

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

#[derive(Serialize, Clone)]
struct Tool {
    r#type: String,
    function: FunctionDef,
}

#[derive(Serialize, Clone)]
struct FunctionDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

fn parse_args() -> (Provider, bool) {
    let args: Vec<String> = std::env::args().collect();
    let mut provider = Provider::Ollama;
    let mut tools_enabled = false;

    for i in 0..args.len().saturating_sub(1) {
        if args[i] == "--provider" && args[i + 1] == "xai" {
            let api_key = std::env::var("XAI_API_KEY")
                .expect("XAI_API_KEY environment variable must be set when using --provider xai");
            let model = std::env::var("XAI_MODEL")
                .unwrap_or_else(|_| "grok-4-1-fast-reasoning".to_string());
            provider = Provider::Xai { api_key, model };
        }
        if args[i] == "--tools" {
            tools_enabled = true;
        }
    }

    (provider, tools_enabled)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lines().map_while(Result::ok).collect();
    let (provider, tools_enabled) = parse_args();
    let response = process_input(&lines, provider, tools_enabled)?;
    println!("{response}");
    Ok(())
}

fn process_input(
    input: &[String],
    provider: Provider,
    tools_enabled: bool,
) -> Result<String, Box<dyn std::error::Error>> {
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

    let tools = if tools_enabled {
        vec![Tool {
            r#type: "function".to_string(),
            function: FunctionDef {
                name: "websearch".to_string(),
                description:
                    "Search the web using SearxNG. Input should be an array of search queries."
                        .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "queries": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Search queries"
                        }
                    },
                    "required": ["queries"]
                }),
            },
        },
        Tool {
            r#type: "function".to_string(),
            function: FunctionDef {
                name: "webfetch".to_string(),
                description: "Fetch the content of URLs and convert to markdown. Use this after a web search when you want to get more detailed content from specific URLs.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "urls": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "URLs to fetch content from"
                        }
                    },
                    "required": ["urls"]
                }),
            },
        }]
    } else {
        vec![]
    };

    let tools = if tools_enabled { Some(tools) } else { None };

    let mut all_messages = messages;
    let final_response = loop {
        let response = chat_with_llm(
            &client,
            all_messages.clone(),
            tools.clone(),
            provider.clone(),
        )?;
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
                    role: "tool".to_string(),
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

    let json = serde_json::to_string(&final_response)?;

    let json = serde_json::to_string(&final_response)?;

    Ok(json)
}

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

fn chat_with_llm(
    client: &reqwest::blocking::Client,
    messages: Vec<Message>,
    tools: Option<Vec<Tool>>,
    provider: Provider,
) -> Result<Message, Box<dyn std::error::Error>> {
    match provider {
        Provider::Ollama => chat_with_ollama(client, messages, tools),
        Provider::Xai { api_key, model } => {
            chat_with_xai(client, messages, tools, &api_key, &model)
        }
    }
}

fn chat_with_ollama(
    client: &reqwest::blocking::Client,
    messages: Vec<Message>,
    tools: Option<Vec<Tool>>,
) -> Result<Message, Box<dyn std::error::Error>> {
    let request = ChatRequest {
        model: "qwen3:8b".to_string(),
        messages,
        tools,
        options: ChatRequestOptions { temperature: 0 },
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

fn chat_with_xai(
    client: &reqwest::blocking::Client,
    messages: Vec<Message>,
    tools: Option<Vec<Tool>>,
    api_key: &str,
    model: &str,
) -> Result<Message, Box<dyn std::error::Error>> {
    let request = ChatRequest {
        model: model.to_string(),
        messages,
        tools,
        options: ChatRequestOptions { temperature: 0 },
        stream: false,
    };

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
