use anyhow::{bail, Result};

use super::FormatRenderer;

/// Renderer for space-delimited key-value pairs (ordered, allows duplicate keys).
///
/// NCL data shape: `[["key", "value"], ...]`
/// Output: `key value\n` per pair
pub struct SpaceDelimitedPairsRenderer;

impl FormatRenderer for SpaceDelimitedPairsRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String> {
        let arr = value
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("SpaceDelimitedPairs expects an array of pairs"))?;

        let mut lines = Vec::new();
        for pair in arr {
            let pair = pair
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("each entry must be a [key, value] array"))?;
            if pair.len() != 2 {
                bail!("each entry must be a [key, value] array of length 2");
            }
            let key = pair[0]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("key must be a string"))?;
            let val = pair[1]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("value must be a string"))?;
            lines.push(format!("{} {}", key, val));
        }
        Ok(lines.join("\n"))
    }

    fn parse(&self, content: &str) -> Result<serde_json::Value> {
        let mut pairs = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = if let Some(pos) = line.find(char::is_whitespace) {
                (&line[..pos], line[pos..].trim())
            } else {
                (line, "")
            };
            pairs.push(serde_json::json!([key, value]));
        }
        Ok(serde_json::Value::Array(pairs))
    }
}

/// Renderer for space-delimited records (alphabetically sorted, no duplicate keys).
///
/// NCL data shape: `{"key": "value", ...}` (Nickel record)
/// Output: `key value\n` per entry
pub struct SpaceDelimitedRecordRenderer;

impl FormatRenderer for SpaceDelimitedRecordRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String> {
        let obj = value
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("SpaceDelimitedRecord expects a JSON object"))?;

        let mut lines = Vec::new();
        for (key, val) in obj {
            let val_str = value_to_string(val);
            lines.push(format!("{} {}", key, val_str));
        }
        Ok(lines.join("\n"))
    }

    fn parse(&self, content: &str) -> Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = if let Some(pos) = line.find(char::is_whitespace) {
                (&line[..pos], line[pos..].trim())
            } else {
                (line, "")
            };
            map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        }
        Ok(serde_json::Value::Object(map))
    }
}

/// Renderer for equal-sign-delimited records (alphabetically sorted, no duplicate keys).
///
/// NCL data shape: `{"key": "value", ...}` (Nickel record)
/// Output: `key=value\n` per entry
pub struct EqualDelimitedRecordRenderer;

impl FormatRenderer for EqualDelimitedRecordRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String> {
        let obj = value
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("EqualDelimitedRecord expects a JSON object"))?;

        let mut lines = Vec::new();
        for (key, val) in obj {
            let val_str = value_to_string(val);
            lines.push(format!("{}={}", key, val_str));
        }
        Ok(lines.join("\n"))
    }

    fn parse(&self, content: &str) -> Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = if let Some(pos) = line.find('=') {
                (line[..pos].trim(), line[pos + 1..].trim())
            } else {
                (line, "")
            };
            map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        }
        Ok(serde_json::Value::Object(map))
    }
}

fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(value_to_string)
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::Object(_) => "[object]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- SpaceDelimitedPairsRenderer ---

    #[test]
    fn test_render_space_delimited_pairs() {
        let renderer = SpaceDelimitedPairsRenderer;
        let value = json!([
            ["font_size", "16"],
            ["map", "ctrl+shift+c copy_to_clipboard"],
            ["map", "ctrl+shift+v paste_from_clipboard"],
        ]);

        let result = renderer.render(&value).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "font_size 16");
        assert_eq!(lines[1], "map ctrl+shift+c copy_to_clipboard");
        assert_eq!(lines[2], "map ctrl+shift+v paste_from_clipboard");
    }

    #[test]
    fn test_parse_space_delimited_pairs() {
        let renderer = SpaceDelimitedPairsRenderer;
        let input = "# Comment\nfont_size 16\nmap ctrl+shift+c copy_to_clipboard\nmap cmd+v paste_from_clipboard\n";

        let result = renderer.parse(input).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], json!(["font_size", "16"]));
        assert_eq!(arr[1], json!(["map", "ctrl+shift+c copy_to_clipboard"]));
        assert_eq!(arr[2], json!(["map", "cmd+v paste_from_clipboard"]));
    }

    // --- SpaceDelimitedRecordRenderer ---

    #[test]
    fn test_render_space_delimited_record() {
        let renderer = SpaceDelimitedRecordRenderer;
        let value = json!({
            "font_size": "14",
            "font_family": "JetBrains Mono",
        });

        let result = renderer.render(&value).unwrap();
        assert!(result.contains("font_size 14"));
        assert!(result.contains("font_family JetBrains Mono"));
    }

    #[test]
    fn test_parse_space_delimited_record() {
        let renderer = SpaceDelimitedRecordRenderer;
        let input = "# Comment\nfont_size 14\nfont_family JetBrains Mono\n";

        let result = renderer.parse(input).unwrap();
        assert_eq!(result["font_size"], "14");
        assert_eq!(result["font_family"], "JetBrains Mono");
    }

    // --- EqualDelimitedRecordRenderer ---

    #[test]
    fn test_render_equal_delimited_record() {
        let renderer = EqualDelimitedRecordRenderer;
        let value = json!({
            "prefix": "~/.local/share/npm",
            "save-exact": "true",
        });

        let result = renderer.render(&value).unwrap();
        assert!(result.contains("prefix=~/.local/share/npm"));
        assert!(result.contains("save-exact=true"));
    }

    #[test]
    fn test_parse_equal_delimited_record() {
        let renderer = EqualDelimitedRecordRenderer;
        let input = "# Comment\nprefix=~/.local/share/npm\nsave-exact=true\n";

        let result = renderer.parse(input).unwrap();
        assert_eq!(result["prefix"], "~/.local/share/npm");
        assert_eq!(result["save-exact"], "true");
    }
}
