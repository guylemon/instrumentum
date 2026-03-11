use serde::{Deserialize, Serialize};
use std::env;
use std::io;
use std::time::Duration;

const SEARXNG_URL: &str = "http://localhost:8888/search";
const TAVILY_URL: &str = "https://api.tavily.com/search";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    Searxng,
    Tavily,
}

#[derive(Debug, Clone, Copy)]
struct Config {
    provider: Provider,
    max_results: i32,
}

#[derive(Deserialize, Serialize)]
struct SearxngResult {
    title: Option<String>,
    url: Option<String>,
    content: Option<String>,
}

#[derive(Deserialize)]
struct SearxngResponse {
    results: Vec<SearxngResult>,
}

#[derive(Serialize)]
struct TavilyRequest {
    query: String,
    max_results: i32,
    search_depth: String,
    include_answer: bool,
    include_raw_content: bool,
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lines().map_while(Result::ok).collect();
    let config = get_config()?;
    let response = process_input(lines, config)?;
    println!("{response}");
    Ok(())
}

fn get_config() -> Result<Config, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut provider = None;
    let mut max_results = 10;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--provider" if i + 1 < args.len() => {
                provider = Some(match args[i + 1].as_str() {
                    "tavily" => {
                        env::var("TAVILY_API_KEY")
                            .map_err(|_| "TAVILY_API_KEY environment variable is not set")?;
                        Provider::Tavily
                    }
                    "searxng" => Provider::Searxng,
                    _ => return Err(format!("Unknown provider: {}", args[i + 1]).into()),
                });
                i += 2;
            }
            "--max-results" if i + 1 < args.len() => {
                max_results = args[i + 1]
                    .parse()
                    .map_err(|_| "Invalid value for --max-results")?;
                i += 2;
            }
            _ => i += 1,
        }
    }

    let provider = match provider {
        Some(p) => p,
        None => {
            if env::var("TAVILY_API_KEY").is_ok() {
                Provider::Tavily
            } else {
                Provider::Searxng
            }
        }
    };

    Ok(Config {
        provider,
        max_results,
    })
}

fn process_input(
    queries: Vec<String>,
    config: Config,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let results = match config.provider {
        Provider::Tavily => {
            let api_key = env::var("TAVILY_API_KEY")?;
            let mut results: Vec<SearxngResult> = vec![];
            for query in queries {
                let r = TavilyRequest {
                    query,
                    max_results: config.max_results,
                    search_depth: "fast".to_string(),
                    include_answer: false,
                    include_raw_content: false,
                };
                let response = client
                    .post(TAVILY_URL)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .header("Content-Type", "application/json")
                    .json(&r)
                    .send()?;
                let text = response.text()?;
                let tavily_resp: TavilyResponse = serde_json::from_str(&text).map_err(|e| {
                    format!(
                        "JSON parse error: {} - body: {}",
                        e,
                        text.chars().take(200).collect::<String>()
                    )
                })?;
                for result in tavily_resp.results {
                    results.push(SearxngResult {
                        title: Some(result.title),
                        url: Some(result.url),
                        content: Some(result.content),
                    });
                }
            }
            results
        }
        Provider::Searxng => {
            let mut results: Vec<SearxngResult> = vec![];
            for query in queries {
                let params = [("q", query), ("format", "json".to_string())];
                let response = client.post(SEARXNG_URL).form(&params).send()?;
                let text = response.text()?;
                let mut searxng_resp: SearxngResponse =
                    serde_json::from_str(&text).map_err(|e| {
                        format!(
                            "JSON parse error: {} - body: {}",
                            e,
                            text.chars().take(200).collect::<String>()
                        )
                    })?;
                results.append(&mut searxng_resp.results);
            }
            results
        }
    };

    let res = serde_json::to_string(&results)?;
    Ok(res)
}
