use console::style;
use indexmap::IndexMap;

use crate::formats::get_renderer;
use crate::nickel::Format;

use super::DiffResult;

/// Type of change detected for a single key path
#[derive(Debug, Clone, PartialEq)]
pub enum KeyChangeType {
    /// Key exists in Source but not in Target
    Added,
    /// Key exists in Target but not in Source
    Removed,
    /// Key exists in both but with different values
    Modified,
}

/// A single key-level change between Source and Target configs
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KeyChange {
    /// Dotted key path, e.g. "section.subsection.key"
    pub path: String,
    /// What kind of change this is
    pub change_type: KeyChangeType,
    /// Value in the Source config (None for Removed)
    pub repo_value: Option<serde_json::Value>,
    /// Value in the Target config (None for Added)
    pub deployed_value: Option<serde_json::Value>,
    /// Formatted display line for this change
    pub display: String,
}

/// Compute per-key diff between two structured configs, returning individual
/// `KeyChange` entries instead of a monolithic string.
///
/// Reuses the same recursive comparison logic as `semantic_diff` but collects
/// structured results.
pub fn semantic_diff_keys(
    format: Format,
    generated: &str,
    deployed: &str,
    ignore_keys: &[String],
) -> Vec<KeyChange> {
    let renderer = get_renderer(format);

    let gen_value = match renderer.parse(generated) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let dep_value = match renderer.parse(deployed) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let gen_filtered = filter_keys(&gen_value, ignore_keys);
    let dep_filtered = filter_keys(&dep_value, ignore_keys);

    let mut changes = Vec::new();
    collect_key_changes(&gen_filtered, &dep_filtered, "", &mut changes);
    changes
}

/// Recursively collect per-key changes between two JSON values
fn collect_key_changes(
    generated: &serde_json::Value,
    deployed: &serde_json::Value,
    path: &str,
    changes: &mut Vec<KeyChange>,
) {
    if generated == deployed {
        return;
    }

    // Treat numbers as equal if their float values match (e.g., 12 == 12.0)
    if let (serde_json::Value::Number(g), serde_json::Value::Number(d)) = (generated, deployed)
        && g.as_f64() == d.as_f64()
    {
        return;
    }

    match (generated, deployed) {
        (serde_json::Value::Object(gen_obj), serde_json::Value::Object(dep_obj)) => {
            // Keys in generated but not in deployed (additions)
            for (key, gen_val) in gen_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if let Some(dep_val) = dep_obj.get(key) {
                    collect_key_changes(gen_val, dep_val, &key_path, changes);
                } else {
                    let display = format_side_diff(&key_path, Some(gen_val), None, None);
                    changes.push(KeyChange {
                        path: key_path,
                        change_type: KeyChangeType::Added,
                        repo_value: Some(gen_val.clone()),
                        deployed_value: None,
                        display,
                    });
                }
            }

            // Keys in deployed but not in generated (removals)
            for (key, dep_val) in dep_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if !gen_obj.contains_key(key) {
                    let display = format_side_diff(&key_path, None, Some(dep_val), None);
                    changes.push(KeyChange {
                        path: key_path,
                        change_type: KeyChangeType::Removed,
                        repo_value: None,
                        deployed_value: Some(dep_val.clone()),
                        display,
                    });
                }
            }
        }
        (serde_json::Value::Array(gen_arr), serde_json::Value::Array(dep_arr)) => {
            if gen_arr != dep_arr {
                let display = format_side_diff(path, Some(generated), Some(deployed), None);
                changes.push(KeyChange {
                    path: path.to_string(),
                    change_type: KeyChangeType::Modified,
                    repo_value: Some(generated.clone()),
                    deployed_value: Some(deployed.clone()),
                    display,
                });
            }
        }
        _ => {
            let display = format_side_diff(path, Some(generated), Some(deployed), None);
            changes.push(KeyChange {
                path: path.to_string(),
                change_type: KeyChangeType::Modified,
                repo_value: Some(generated.clone()),
                deployed_value: Some(deployed.clone()),
                display,
            });
        }
    }
}

fn format_side_diff(
    path: &str,
    source: Option<&serde_json::Value>,
    target: Option<&serde_json::Value>,
    base: Option<Option<&serde_json::Value>>,
) -> String {
    let mut lines = vec![
        format!("{} {}", style("~").yellow(), style(path).yellow()),
        format!("  {}", style(side_value_line("<< Source", source)).blue()),
        format!(
            "  {}",
            style(side_value_line(">> Target", target)).magenta()
        ),
    ];
    if let Some(base) = base {
        lines.push(format!(
            "  {}",
            style(side_value_line("|| Base", base)).color256(8)
        ));
    }
    lines.join("\n")
}

/// Return a copy of `change` whose display includes the Base side from
/// `base_root`. Passing `None` preserves the existing 2-way display.
pub fn key_change_with_base_display(
    change: &KeyChange,
    base_root: Option<&serde_json::Value>,
) -> KeyChange {
    let Some(base_root) = base_root else {
        return change.clone();
    };
    let mut change = change.clone();
    change.display = format_side_diff(
        &change.path,
        change.repo_value.as_ref(),
        change.deployed_value.as_ref(),
        Some(lookup_dotted(base_root, &change.path)),
    );
    change
}

fn lookup_dotted<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(value);
    }
    let mut cur = value;
    for segment in path.split('.') {
        cur = cur.as_object()?.get(segment)?;
    }
    Some(cur)
}

fn side_value_line(label: &str, value: Option<&serde_json::Value>) -> String {
    let value = value
        .map(format_value)
        .unwrap_or_else(|| "<missing>".to_string());
    format!("{label}: {value}")
}

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

/// Perform semantic diff on structured configs and include Base values when
/// the Base document parses successfully.
pub fn semantic_diff_with_base(
    format: Format,
    generated: &str,
    deployed: &str,
    base: &str,
    ignore_keys: &[String],
) -> DiffResult {
    let renderer = get_renderer(format);

    let gen_value = match renderer.parse(generated) {
        Ok(v) => v,
        Err(_) => return super::text::text_diff_with_base(generated, deployed, base, &[]),
    };

    let dep_value = match renderer.parse(deployed) {
        Ok(v) => v,
        Err(_) => return super::text::text_diff_with_base(generated, deployed, base, &[]),
    };

    let base_value = match renderer.parse(base) {
        Ok(v) => v,
        Err(_) => return semantic_diff(format, generated, deployed, ignore_keys),
    };

    let gen_filtered = filter_keys(&gen_value, ignore_keys);
    let dep_filtered = filter_keys(&dep_value, ignore_keys);
    let base_filtered = filter_keys(&base_value, ignore_keys);

    let mut output = Vec::new();
    let has_changes = diff_values_with_base(
        &gen_filtered,
        &dep_filtered,
        Some(&base_filtered),
        true,
        "",
        &mut output,
    );

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

    // Treat numbers as equal if their float values match (e.g., 12 == 12.0)
    if let (serde_json::Value::Number(g), serde_json::Value::Number(d)) = (generated, deployed)
        && g.as_f64() == d.as_f64()
    {
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
                    format!("{path}.{key}")
                };

                if let Some(dep_val) = dep_obj.get(key) {
                    if diff_values(gen_val, dep_val, &key_path, output) {
                        has_changes = true;
                    }
                } else {
                    output.push(format_side_diff(&key_path, Some(gen_val), None, None));
                    has_changes = true;
                }
            }

            // Keys in deployed but not in generated (deletions)
            for (key, dep_val) in dep_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if !gen_obj.contains_key(key) {
                    output.push(format_side_diff(&key_path, None, Some(dep_val), None));
                    has_changes = true;
                }
            }

            has_changes
        }
        (serde_json::Value::Array(gen_arr), serde_json::Value::Array(dep_arr)) => {
            if gen_arr != dep_arr {
                output.push(format_side_diff(
                    path,
                    Some(generated),
                    Some(deployed),
                    None,
                ));
                true
            } else {
                false
            }
        }
        _ => {
            output.push(format_side_diff(
                path,
                Some(generated),
                Some(deployed),
                None,
            ));
            true
        }
    }
}

fn child_base<'a>(base: Option<&'a serde_json::Value>, key: &str) -> Option<&'a serde_json::Value> {
    base.and_then(|v| v.as_object())
        .and_then(|obj| obj.get(key))
}

fn diff_values_with_base(
    generated: &serde_json::Value,
    deployed: &serde_json::Value,
    base: Option<&serde_json::Value>,
    show_base: bool,
    path: &str,
    output: &mut Vec<String>,
) -> bool {
    if generated == deployed {
        return false;
    }

    if let (serde_json::Value::Number(g), serde_json::Value::Number(d)) = (generated, deployed)
        && g.as_f64() == d.as_f64()
    {
        return false;
    }

    match (generated, deployed) {
        (serde_json::Value::Object(gen_obj), serde_json::Value::Object(dep_obj)) => {
            let mut has_changes = false;

            for (key, gen_val) in gen_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                let base_val = child_base(base, key);

                if let Some(dep_val) = dep_obj.get(key) {
                    if diff_values_with_base(
                        gen_val, dep_val, base_val, show_base, &key_path, output,
                    ) {
                        has_changes = true;
                    }
                } else {
                    output.push(format_side_diff(
                        &key_path,
                        Some(gen_val),
                        None,
                        if show_base { Some(base_val) } else { None },
                    ));
                    has_changes = true;
                }
            }

            for (key, dep_val) in dep_obj {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if !gen_obj.contains_key(key) {
                    output.push(format_side_diff(
                        &key_path,
                        None,
                        Some(dep_val),
                        if show_base {
                            Some(child_base(base, key))
                        } else {
                            None
                        },
                    ));
                    has_changes = true;
                }
            }

            has_changes
        }
        (serde_json::Value::Array(gen_arr), serde_json::Value::Array(dep_arr)) => {
            if gen_arr != dep_arr {
                output.push(format_side_diff(
                    path,
                    Some(generated),
                    Some(deployed),
                    if show_base { Some(base) } else { None },
                ));
                true
            } else {
                false
            }
        }
        _ => {
            output.push(format_side_diff(
                path,
                Some(generated),
                Some(deployed),
                if show_base { Some(base) } else { None },
            ));
            true
        }
    }
}

fn format_value(value: &serde_json::Value) -> String {
    format_value_with_depth(value, 0)
}

fn format_value_with_depth(value: &serde_json::Value, depth: usize) -> String {
    match value {
        serde_json::Value::String(s) => format!("\"{s}\""),
        serde_json::Value::Array(arr) => format_array(arr, depth),
        serde_json::Value::Object(obj) => format_object(obj, depth),
        _ => value.to_string(),
    }
}

fn format_object(obj: &serde_json::Map<String, serde_json::Value>, depth: usize) -> String {
    if obj.is_empty() {
        return "{}".to_string();
    }
    if depth >= 3 {
        return format!("{{...{} fields}}", obj.len());
    }

    const MAX_INLINE_OBJECT_FIELDS: usize = 4;
    let mut items: Vec<_> = obj
        .iter()
        .take(MAX_INLINE_OBJECT_FIELDS)
        .map(|(key, value)| {
            format!(
                "{}: {}",
                serde_json::to_string(key).unwrap_or_else(|_| format!("\"{key}\"")),
                format_value_with_depth(value, depth + 1)
            )
        })
        .collect();

    let remaining = obj.len().saturating_sub(MAX_INLINE_OBJECT_FIELDS);
    if remaining > 0 {
        items.push(format!("...{remaining} more"));
    }

    format!("{{{}}}", items.join(", "))
}

fn format_array(arr: &[serde_json::Value], depth: usize) -> String {
    if arr.len() <= 3 {
        let items: Vec<_> = arr
            .iter()
            .map(|value| format_value_with_depth(value, depth + 1))
            .collect();
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
        assert!(result.output.contains("<< Source: \"new\""));
        assert!(result.output.contains(">> Target: \"old\""));
    }

    #[test]
    fn test_semantic_diff_source_only_object_expands_fields() {
        let generated =
            r#"{"terminal": {"shell": {"program": "/usr/bin/env", "args": ["elvish"]}}}"#;
        let deployed = r#"{}"#;
        let result = semantic_diff(Format::Json, generated, deployed, &[]);
        assert!(result.has_changes);
        assert!(result.output.contains("~ terminal"));
        assert!(result.output.contains("\"shell\""));
        assert!(result.output.contains("\"program\": \"/usr/bin/env\""));
        assert!(result.output.contains("\"args\": [\"elvish\"]"));
        assert!(!result.output.contains("<< Source: {...}"));
    }

    #[test]
    fn test_semantic_diff_with_base_shows_base_side() {
        let generated = r#"{"key": "new"}"#;
        let deployed = r#"{"key": "old"}"#;
        let base = r#"{"key": "original"}"#;
        let result = semantic_diff_with_base(Format::Json, generated, deployed, base, &[]);
        assert!(result.has_changes);
        assert!(result.output.contains("<< Source: \"new\""));
        assert!(result.output.contains(">> Target: \"old\""));
        assert!(result.output.contains("|| Base: \"original\""));
    }

    #[test]
    fn test_semantic_diff_keys_no_changes() {
        let config = r#"{"key": "value"}"#;
        let changes = semantic_diff_keys(Format::Json, config, config, &[]);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_semantic_diff_keys_modified() {
        let generated = r#"{"key": "new", "other": 1}"#;
        let deployed = r#"{"key": "old", "other": 1}"#;
        let changes = semantic_diff_keys(Format::Json, generated, deployed, &[]);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "key");
        assert_eq!(changes[0].change_type, KeyChangeType::Modified);
        assert_eq!(changes[0].repo_value, Some(json!("new")));
        assert_eq!(changes[0].deployed_value, Some(json!("old")));
        assert!(changes[0].display.contains("<< Source: \"new\""));
        assert!(changes[0].display.contains(">> Target: \"old\""));
    }

    #[test]
    fn test_key_change_with_base_display_shows_base_side() {
        let generated = r#"{"key": "new"}"#;
        let deployed = r#"{"key": "old"}"#;
        let changes = semantic_diff_keys(Format::Json, generated, deployed, &[]);
        let base = json!({"key": "original"});
        let change = key_change_with_base_display(&changes[0], Some(&base));
        assert!(change.display.contains("<< Source: \"new\""));
        assert!(change.display.contains(">> Target: \"old\""));
        assert!(change.display.contains("|| Base: \"original\""));
    }

    #[test]
    fn test_semantic_diff_keys_added_and_removed() {
        let generated = r#"{"repo_only": 1, "shared": true}"#;
        let deployed = r#"{"deployed_only": 2, "shared": true}"#;
        let changes = semantic_diff_keys(Format::Json, generated, deployed, &[]);
        assert_eq!(changes.len(), 2);

        let added = changes.iter().find(|c| c.path == "repo_only").unwrap();
        assert_eq!(added.change_type, KeyChangeType::Added);
        assert!(added.deployed_value.is_none());

        let removed = changes.iter().find(|c| c.path == "deployed_only").unwrap();
        assert_eq!(removed.change_type, KeyChangeType::Removed);
        assert!(removed.repo_value.is_none());
    }

    #[test]
    fn test_semantic_diff_keys_nested() {
        let generated = r#"{"section": {"a": 1, "b": 2}}"#;
        let deployed = r#"{"section": {"a": 1, "b": 3}}"#;
        let changes = semantic_diff_keys(Format::Json, generated, deployed, &[]);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "section.b");
        assert_eq!(changes[0].change_type, KeyChangeType::Modified);
    }

    #[test]
    fn test_semantic_diff_keys_ignores_keys() {
        let generated = r#"{"keep": "new", "skip": "new"}"#;
        let deployed = r#"{"keep": "old", "skip": "old"}"#;
        let changes = semantic_diff_keys(Format::Json, generated, deployed, &["skip".to_string()]);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "keep");
    }
}
