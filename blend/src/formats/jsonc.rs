use anyhow::{Context, Result};

use super::FormatRenderer;

pub struct JsoncRenderer;

/// Strip JSONC comments and trailing commas, then parse as standard JSON.
pub fn parse_jsonc(content: &str) -> Result<serde_json::Value> {
    let mut stripped = content.to_string();
    json_strip_comments::strip(&mut stripped).context("Failed to strip JSONC comments")?;
    serde_json::from_str(&stripped).context("Failed to parse JSONC (after stripping comments)")
}

impl FormatRenderer for JsoncRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String> {
        // JSONC render outputs standard JSON — comments belong in .ncl source, not deployed files
        serde_json::to_string_pretty(value).context("Failed to serialize to JSON")
    }

    fn parse(&self, content: &str) -> Result<serde_json::Value> {
        parse_jsonc(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_line_comments() {
        let renderer = JsoncRenderer;
        let input = r#"{
            // This is a line comment
            "editor.fontSize": 14,
            "editor.fontFamily": "JetBrains Mono" // trailing comment
        }"#;

        let result = renderer.parse(input).unwrap();
        assert_eq!(result["editor.fontSize"], 14);
        assert_eq!(result["editor.fontFamily"], "JetBrains Mono");
    }

    #[test]
    fn test_parse_block_comments() {
        let renderer = JsoncRenderer;
        let input = r#"{
            /* Block comment */
            "key": "value",
            "nested": {
                /* multi
                   line
                   comment */
                "inner": true
            }
        }"#;

        let result = renderer.parse(input).unwrap();
        assert_eq!(result["key"], "value");
        assert_eq!(result["nested"]["inner"], true);
    }

    #[test]
    fn test_parse_trailing_commas() {
        let renderer = JsoncRenderer;
        let input = r#"{
            "a": 1,
            "b": [1, 2, 3,],
            "c": {"x": true,},
        }"#;

        let result = renderer.parse(input).unwrap();
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], json!([1, 2, 3]));
        assert_eq!(result["c"]["x"], true);
    }

    #[test]
    fn test_parse_combined() {
        let renderer = JsoncRenderer;
        let input = r#"{
            // VS Code settings
            "editor.fontSize": 14,
            /* font config */
            "editor.fontFamily": "JetBrains Mono",
            "editor.tabSize": 4, // spaces
        }"#;

        let result = renderer.parse(input).unwrap();
        assert_eq!(result["editor.fontSize"], 14);
        assert_eq!(result["editor.fontFamily"], "JetBrains Mono");
        assert_eq!(result["editor.tabSize"], 4);
    }

    #[test]
    fn test_render_standard_json() {
        let renderer = JsoncRenderer;
        let value = json!({
            "editor.fontSize": 14,
            "editor.fontFamily": "JetBrains Mono"
        });

        let result = renderer.render(&value).unwrap();
        // Output must be valid standard JSON (no comments, no trailing commas)
        let reparsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(reparsed["editor.fontSize"], 14);
        assert_eq!(reparsed["editor.fontFamily"], "JetBrains Mono");
        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));
    }

    #[test]
    fn test_parse_plain_json() {
        // JSONC parser should handle plain JSON just fine
        let renderer = JsoncRenderer;
        let input = r#"{"key": "value", "num": 42}"#;

        let result = renderer.parse(input).unwrap();
        assert_eq!(result["key"], "value");
        assert_eq!(result["num"], 42);
    }
}
