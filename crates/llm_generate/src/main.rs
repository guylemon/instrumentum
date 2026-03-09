use llm_msg::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self};
use std::time::Duration;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    tools: Option<Vec<Tool>>,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: Option<Message>,
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

    let tools = vec![Tool {
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
    }];

    let mut all_messages = messages;
    let final_response = loop {
        let response = chat_with_llm(&client, all_messages.clone(), Some(tools.clone()))?;
        all_messages.push(response.clone());

        if let Some(ref tool_calls) = response.tool_calls {
            if tool_calls.is_empty() {
                break response;
            }

            for tc in tool_calls {
                let name = tc.function.name.clone();
                let args = tc.function.arguments.clone();
                eprintln!("...calling tool: {name}({args:?})");

                let result = execute_tool(&name, &args)?;
                eprintln!("...result: {result}");
                all_messages.push(Message {
                    role: "tool".to_string(),
                    content: result,
                    tool_name: Some(name),
                    tool_calls: None,
                });
            }
        } else {
            break response;
        }
    };

    let json = serde_json::to_string(&final_response)?;

    Ok(json)
}

fn execute_tool(
    name: &str,
    args: &serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
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

            use std::io::Write;
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
) -> Result<Message, Box<dyn std::error::Error>> {
    let request = ChatRequest {
        model: "granite4:3b".to_string(),
        messages,
        tools,
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
