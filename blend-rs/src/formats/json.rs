use anyhow::{Context, Result};

use super::FormatRenderer;

pub struct JsonRenderer;

impl FormatRenderer for JsonRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String> {
        serde_json::to_string_pretty(value)
            .context("Failed to serialize to JSON")
    }

    fn parse(&self, content: &str) -> Result<serde_json::Value> {
        serde_json::from_str(content)
            .context("Failed to parse JSON")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_json() {
        let renderer = JsonRenderer;
        let value = json!({
            "editor.fontSize": 14,
            "editor.fontFamily": "JetBrains Mono"
        });

        let result = renderer.render(&value).unwrap();
        assert!(result.contains("editor.fontSize"));
        assert!(result.contains("14"));
    }

    #[test]
    fn test_parse_json() {
        let renderer = JsonRenderer;
        let json = r#"{"editor.fontSize": 14}"#;

        let result = renderer.parse(json).unwrap();
        assert_eq!(result["editor.fontSize"], 14);
    }
}
