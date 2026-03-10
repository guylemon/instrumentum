use std::io;
use std::path::PathBuf;

const MAX_REFERENCE_CHARS: usize = 60000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let urls: Vec<String> = stdin.lines().map_while(Result::ok).collect();
    let mut response: String = String::new();
    let mut failed_urls: Vec<String> = Vec::new();
    for url in urls {
        match fetch_html(&url) {
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
    Ok(())
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
