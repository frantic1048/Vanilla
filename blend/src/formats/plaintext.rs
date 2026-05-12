use anyhow::Result;

use super::FormatRenderer;

/// Renderer for plaintext files (no transformation)
/// Used primarily for source directory copies
pub struct PlaintextRenderer;

impl FormatRenderer for PlaintextRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String> {
        // For plaintext, we expect the value to be a string
        match value {
            serde_json::Value::String(s) => Ok(s.clone()),
            _ => Ok(serde_json::to_string_pretty(value)?),
        }
    }

    fn parse(&self, content: &str) -> Result<serde_json::Value> {
        // Return content as a string value
        Ok(serde_json::Value::String(content.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_string() {
        let renderer = PlaintextRenderer;
        let value = json!("vim.opt.number = true");
        let result = renderer.render(&value).unwrap();
        assert_eq!(result, "vim.opt.number = true");
    }

    #[test]
    fn test_parse_plaintext() {
        let renderer = PlaintextRenderer;
        let content = "some plain text content";
        let result = renderer.parse(content).unwrap();
        assert_eq!(result, json!("some plain text content"));
    }
}
