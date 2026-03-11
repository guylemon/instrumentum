use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

const SEARXNG_URL: &str = "http://localhost:8888/search";

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lines().map_while(Result::ok).collect();
    let response = process_input(lines)?;
    println!("{response}");
    Ok(())
}

fn process_input(queries: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
    let mut results: Vec<SearxngResult> = vec![];
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    for query in queries {
        let params = [("q", query), ("format", "json".to_string())];
        let response = client.post(SEARXNG_URL).form(&params).send()?;
        let text = response.text()?;
        let mut searxng_resp: SearxngResponse = serde_json::from_str(&text).map_err(|e| {
            format!(
                "JSON parse error: {} - body: {}",
                e,
                text.chars().take(200).collect::<String>()
            )
        })?;

        results.append(&mut searxng_resp.results);
    }

    let res = serde_json::to_string(&results)?;
    Ok(res)
}
