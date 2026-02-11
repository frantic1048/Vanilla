use colored::Colorize;
use indexmap::IndexMap;

use crate::formats::get_renderer;
use crate::nickel::Format;

use super::DiffResult;

/// Perform semantic diff on structured configs (JSON, TOML, YAML)
pub fn semantic_diff(
    format: Format,
    generated: &str,
    deployed: &str,
    ignore_keys: &[String],
) -> DiffResult {
    let renderer = get_renderer(format);

    let gen_value = match renderer.parse(generated) {
        Ok(v) => v,
        Err(_) => return super::text::text_diff(generated, deployed, &[]),
    };

    let dep_value = match renderer.parse(deployed) {
        Ok(v) => v,
        Err(_) => return super::text::text_diff(generated, deployed, &[]),
    };

    // Filter out ignored keys
    let gen_filtered = filter_keys(&gen_value, ignore_keys);
    let dep_filtered = filter_keys(&dep_value, ignore_keys);

    // Compare
    let mut output = Vec::new();
    let has_changes = diff_values(&gen_filtered, &dep_filtered, "", &mut output);

    if has_changes {
        DiffResult::with_changes(output.join("\n"))
    } else {
        DiffResult::no_changes()
    }
}

/// Filter out ignored keys from a JSON value
fn filter_keys(value: &serde_json::Value, ignore_keys: &[String]) -> serde_json::Value {
    match value {
        serde_json::Value::Object(obj) => {
            let filtered: IndexMap<String, serde_json::Value> = obj
                .iter()
                .filter(|(k, _)| !ignore_keys.contains(k))
                .map(|(k, v)| (k.clone(), filter_keys(v, ignore_keys)))
                .collect();
            serde_json::Value::Object(filtered.into_iter().collect())
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(|v| filter_keys(v, ignore_keys)).collect())
        }
        _ => value.clone(),
    }
}

/// Recursively diff two JSON values
fn diff_values(
    generated: &serde_json::Value,
    deployed: &serde_json::Value,
    path: &str,
    output: &mut Vec<String>,
) -> bool {
    if generated == deployed {
        return false;
    }

    match (generated, deployed) {
        (serde_json::Value::Object(gen_obj), serde_json::Value::Object(dep_obj)) => {
            let mut has_changes = false;

            // Keys in generated but not in deployed (additions)
            for (key, gen_val) in gen_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                if let Some(dep_val) = dep_obj.get(key) {
                    if diff_values(gen_val, dep_val, &key_path, output) {
                        has_changes = true;
                    }
                } else {
                    output.push(format!(
                        "{} {} = {}",
                        "+".green(),
                        key_path.green(),
                        format_value(gen_val).green()
                    ));
                    has_changes = true;
                }
            }

            // Keys in deployed but not in generated (deletions)
            for (key, dep_val) in dep_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                if !gen_obj.contains_key(key) {
                    output.push(format!(
                        "{} {} = {}",
                        "-".red(),
                        key_path.red(),
                        format_value(dep_val).red()
                    ));
                    has_changes = true;
                }
            }

            has_changes
        }
        (serde_json::Value::Array(gen_arr), serde_json::Value::Array(dep_arr)) => {
            if gen_arr != dep_arr {
                output.push(format!(
                    "{} {} = {}",
                    "~".yellow(),
                    path.yellow(),
                    format!("{} -> {}", format_array(dep_arr), format_array(gen_arr)).yellow()
                ));
                true
            } else {
                false
            }
        }
        _ => {
            output.push(format!(
                "{} {} = {} {} {}",
                "~".yellow(),
                path.yellow(),
                format_value(deployed).red(),
                "->".dimmed(),
                format_value(generated).green()
            ));
            true
        }
    }
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => format!("\"{}\"", s),
        serde_json::Value::Array(arr) => format_array(arr),
        serde_json::Value::Object(_) => "{...}".to_string(),
        _ => value.to_string(),
    }
}

fn format_array(arr: &[serde_json::Value]) -> String {
    if arr.len() <= 3 {
        let items: Vec<_> = arr.iter().map(format_value).collect();
        format!("[{}]", items.join(", "))
    } else {
        format!("[...{} items]", arr.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_keys() {
        let value = json!({
            "keep": "this",
            "remove": "this",
            "nested": {
                "keep": "nested",
                "remove": "also"
            }
        });

        let filtered = filter_keys(&value, &["remove".to_string()]);
        assert!(filtered.get("keep").is_some());
        assert!(filtered.get("remove").is_none());
        assert!(filtered["nested"].get("keep").is_some());
        assert!(filtered["nested"].get("remove").is_none());
    }

    #[test]
    fn test_semantic_diff_no_changes() {
        let config = r#"{"key": "value"}"#;
        let result = semantic_diff(Format::Json, config, config, &[]);
        assert!(!result.has_changes);
    }

    #[test]
    fn test_semantic_diff_with_changes() {
        let generated = r#"{"key": "new"}"#;
        let deployed = r#"{"key": "old"}"#;
        let result = semantic_diff(Format::Json, generated, deployed, &[]);
        assert!(result.has_changes);
        assert!(result.output.contains("key"));
    }
}
