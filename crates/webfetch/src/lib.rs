use llm_provider::FunctionDef;
use llm_provider::Tool;
use serde_json::json;

#[must_use]
pub fn tool_def() -> Tool {
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
    }
}
