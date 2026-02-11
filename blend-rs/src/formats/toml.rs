use anyhow::{Context, Result};
use indexmap::IndexMap;

use super::FormatRenderer;

pub struct TomlRenderer;

impl FormatRenderer for TomlRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String> {
        // Convert JSON to TOML-compatible structure
        let toml_value = json_to_toml(value)?;
        let output = ::toml::to_string_pretty(&toml_value)
            .context("Failed to serialize to TOML")?;
        Ok(output)
    }

    fn parse(&self, content: &str) -> Result<serde_json::Value> {
        let toml_value: ::toml::Value = content
            .parse()
            .context("Failed to parse TOML")?;
        let json = toml_to_json(&toml_value);
        Ok(json)
    }
}

/// Convert JSON Value to TOML Value
fn json_to_toml(json: &serde_json::Value) -> Result<::toml::Value> {
    match json {
        serde_json::Value::Null => Ok(::toml::Value::String("null".to_string())),
        serde_json::Value::Bool(b) => Ok(::toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(::toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(::toml::Value::Float(f))
            } else {
                Ok(::toml::Value::String(n.to_string()))
            }
        }
        serde_json::Value::String(s) => Ok(::toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<_>> = arr.iter().map(json_to_toml).collect();
            Ok(::toml::Value::Array(values?))
        }
        serde_json::Value::Object(obj) => {
            let mut map = ::toml::map::Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_toml(v)?);
            }
            Ok(::toml::Value::Table(map))
        }
    }
}

/// Convert TOML Value to JSON Value
fn toml_to_json(toml: &::toml::Value) -> serde_json::Value {
    match toml {
        ::toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        ::toml::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        ::toml::Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        ::toml::Value::String(s) => serde_json::Value::String(s.clone()),
        ::toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        ::toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(toml_to_json).collect())
        }
        ::toml::Value::Table(table) => {
            let map: IndexMap<String, serde_json::Value> = table
                .iter()
                .map(|(k, v)| (k.clone(), toml_to_json(v)))
                .collect();
            serde_json::Value::Object(map.into_iter().collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_toml() {
        let renderer = TomlRenderer;
        let value = json!({
            "format": "$character$directory",
            "character": {
                "success_symbol": "[>](bold green)"
            }
        });

        let result = renderer.render(&value).unwrap();
        assert!(result.contains("format = "));
        assert!(result.contains("[character]"));
    }

    #[test]
    fn test_parse_toml() {
        let renderer = TomlRenderer;
        let toml = r#"
format = "$character"

[character]
success_symbol = "[>](bold green)"
"#;

        let result = renderer.parse(toml).unwrap();
        assert_eq!(result["format"], "$character");
        assert_eq!(result["character"]["success_symbol"], "[>](bold green)");
    }

    #[test]
    fn test_roundtrip() {
        let renderer = TomlRenderer;
        let original = json!({
            "key": "value",
            "number": 42,
            "nested": {
                "bool": true
            }
        });

        let toml = renderer.render(&original).unwrap();
        let parsed = renderer.parse(&toml).unwrap();

        assert_eq!(original["key"], parsed["key"]);
        assert_eq!(original["number"], parsed["number"]);
        assert_eq!(original["nested"]["bool"], parsed["nested"]["bool"]);
    }
}
