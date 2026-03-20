use serde_json::json;

use llm_provider::FunctionDef;
use llm_provider::Tool;

#[must_use]
pub fn tool_def() -> Tool {
    Tool {
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
    }
}
