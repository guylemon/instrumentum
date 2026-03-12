use std::path::PathBuf;
use std::{env, io};

const MAX_REFERENCE_CHARS: usize = 60000;

fn main() {
    let args: Vec<String> = env::args().collect();
    let use_tavily = args
        .iter()
        .any(|arg| arg == "--provider" && args.iter().any(|a| a == "tavily"));

    let stdin = io::stdin();
    let urls: Vec<String> = stdin.lines().map_while(Result::ok).collect();
    let mut response: String = String::new();
    let mut failed_urls: Vec<String> = Vec::new();
    for url in urls {
        let fetch_fn: fn(&str) -> Result<String, Box<dyn std::error::Error>> =
            if use_tavily { fetch_tavily } else { fetch_html };
        match fetch_fn(&url) {
            Ok(markdown) => {
                response.push_str(format!("\n\n--url:{url}\n\n").as_str());
                response.push_str(markdown.as_str());
            }
            Err(e) => {
                eprintln!("Warning: Failed to fetch {url}: {e}");
                failed_urls.push(url);
            }
        }
    }
    if !failed_urls.is_empty() {
        eprintln!("Warning: {} URL(s) failed to fetch", failed_urls.len());
    }
    println!("{response}");
}

fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("instrumentum/webfetch")
}

fn get_cache_path(url: &str) -> PathBuf {
    let hash = md5::compute(url);
    get_cache_dir().join(format!("{hash:x}"))
}

fn get_cached(url: &str) -> Option<String> {
    std::fs::read_to_string(get_cache_path(url)).ok()
}

fn cache_result(url: &str, markdown: &str) {
    let dir = get_cache_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("Warning: Failed to create cache directory: {e}");
        return;
    }

    if let Err(e) = std::fs::write(get_cache_path(url), markdown) {
        eprintln!("Warning: Failed to write cache for {url}: {e}");
    }
}

fn fetch_html(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(cached) = get_cached(url) {
        return Ok(cached);
    }

    let readable = readability::extractor::scrape(url)?;

    let content = String::from_utf8_lossy(readable.content.as_bytes()).into_owned();

    if content.is_empty() {
        return Ok(String::new());
    }

    let truncated = if content.chars().count() > MAX_REFERENCE_CHARS {
        content
            .chars()
            .take(MAX_REFERENCE_CHARS)
            .collect::<String>()
    } else {
        content
    };

    let md_options = html_to_markdown_rs::ConversionOptions {
        skip_images: true,
        strip_tags: vec![
            "style".to_string(),
            "meta".to_string(),
            "script".to_string(),
            "noscript".to_string(),
            "iframe".to_string(),
            "object".to_string(),
            "embed".to_string(),
        ],
        ..Default::default()
    };

    let markdown = match html_to_markdown_rs::convert(&truncated, Some(md_options)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: html-to-markdown failed for {url}: {e}");
            return Ok(String::new());
        }
    };

    cache_result(url, &markdown);
    Ok(markdown)
}

#[derive(serde::Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
    failed_results: Vec<TavilyFailedResult>,
}

#[derive(serde::Deserialize)]
struct TavilyResult {
    #[allow(dead_code)]
    url: String,
    raw_content: String,
}

#[derive(serde::Deserialize)]
struct TavilyFailedResult {
    url: String,
    error: String,
}

fn fetch_tavily(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(cached) = get_cached(url) {
        return Ok(cached);
    }

    let api_key =
        env::var("TAVILY_API_KEY").map_err(|_| "TAVILY_API_KEY environment variable not set")?;

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.tavily.com/extract")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&serde_json::json!({
            "urls": [url],
            "format": "markdown"
        }))
        .send()?;

    if !response.status().is_success() {
        return Err(format!("Tavily API error: {}", response.status()).into());
    }

    let tavily_response: TavilyResponse = response.json()?;

    if !tavily_response.failed_results.is_empty() {
        for failed in &tavily_response.failed_results {
            eprintln!(
                "Warning: Tavily failed to extract {}: {}",
                failed.url, failed.error
            );
        }
    }

    let content = tavily_response
        .results
        .first()
        .ok_or("No results returned from Tavily")?
        .raw_content
        .clone();

    let truncated = if content.chars().count() > MAX_REFERENCE_CHARS {
        content.chars().take(MAX_REFERENCE_CHARS).collect()
    } else {
        content
    };

    cache_result(url, &truncated);
    Ok(truncated)
}
