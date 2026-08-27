use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result, anyhow, bail};
use serde_saphyr::granit_parser::{Event, Parser, ScalarStyle, Span, Tag};

use super::FormatRenderer;

const MAX_YAML_EVENTS: usize = 100_000;
const MAX_YAML_NODES: usize = 100_000;
const MAX_YAML_DEPTH: usize = 128;
const MAX_YAML_ALIASES: usize = 1_000;

/// YAML 1.2 renderer/parser for Blend's JSON-compatible value model.
pub struct YamlRenderer;

#[derive(Clone, Debug)]
enum YamlNode {
    Scalar {
        value: String,
        style: ScalarStyle,
        tag: Option<Tag>,
        span: Span,
    },
    Sequence {
        values: Vec<YamlNode>,
        tag: Option<Tag>,
        span: Span,
    },
    Mapping {
        entries: Vec<(YamlNode, YamlNode)>,
        tag: Option<Tag>,
        span: Span,
    },
}

#[derive(Debug)]
enum Frame {
    Sequence {
        values: Vec<YamlNode>,
        anchor: usize,
        tag: Option<Tag>,
        span: Span,
    },
    Mapping {
        entries: Vec<(YamlNode, YamlNode)>,
        pending_key: Option<Box<YamlNode>>,
        anchor: usize,
        tag: Option<Tag>,
        span: Span,
    },
}

#[derive(Clone, Debug)]
struct Anchor {
    node: YamlNode,
    nodes: usize,
    depth: usize,
}

#[derive(Default)]
struct ParseBudget {
    events: usize,
    nodes: usize,
    aliases: usize,
}

impl ParseBudget {
    fn event(&mut self, span: Span) -> Result<()> {
        self.events += 1;
        if self.events > MAX_YAML_EVENTS {
            return Err(located_error(span, "YAML event limit exceeded"));
        }
        Ok(())
    }

    fn nodes(&mut self, count: usize, span: Span) -> Result<()> {
        self.nodes = self
            .nodes
            .checked_add(count)
            .ok_or_else(|| located_error(span, "YAML node limit exceeded"))?;
        if self.nodes > MAX_YAML_NODES {
            return Err(located_error(span, "YAML node limit exceeded"));
        }
        Ok(())
    }

    fn alias(&mut self, anchor: &Anchor, parent_depth: usize, span: Span) -> Result<()> {
        self.aliases += 1;
        if self.aliases > MAX_YAML_ALIASES {
            return Err(located_error(span, "YAML alias limit exceeded"));
        }
        ensure_depth(parent_depth + anchor.depth, span)?;
        self.nodes(anchor.nodes, span)
    }
}

impl FormatRenderer for YamlRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String> {
        let options = serde_saphyr::ser_options! {
            indent_step: 2,
            compact_list_indent: false,
            prefer_block_scalars: true,
            quote_all: false,
            // The library's YAML 1.2 switch also emits a directive. Leaving it
            // disabled produces conservative quoting without a document header.
            yaml_12: false,
        };
        let output = serde_saphyr::to_string_with_options(value, options)
            .context("Failed to serialize YAML")?;
        if output.starts_with("%YAML") || output.starts_with("---") {
            bail!("YAML serializer emitted an unexpected document directive or marker");
        }
        Ok(output)
    }

    fn parse(&self, content: &str) -> Result<serde_json::Value> {
        let node = parse_document(content)?;
        node_to_json(&node, "$", false, 1)
    }
}

fn parse_document(content: &str) -> Result<YamlNode> {
    if content.trim().is_empty() {
        bail!("Failed to parse YAML: empty YAML stream");
    }

    let mut anchors = HashMap::<usize, Anchor>::new();
    let mut frames = Vec::<Frame>::new();
    let mut root = None;
    let mut documents = 0usize;
    let mut budget = ParseBudget::default();

    for next in Parser::new_from_str(content) {
        let (event, span) = next.map_err(|error| anyhow!("Failed to parse YAML: {error}"))?;
        budget.event(span)?;
        match event {
            Event::StreamStart | Event::StreamEnd | Event::Comment(..) => {}
            Event::DocumentStart(_, version) => {
                documents += 1;
                if documents > 1 {
                    bail!("Failed to parse YAML: expected exactly one document");
                }
                if version.is_some_and(|version| version.major != 1 || version.minor != 2) {
                    bail!("Failed to parse YAML: only YAML 1.2 documents are supported");
                }
            }
            Event::DocumentEnd => {
                if !frames.is_empty() {
                    bail!("Failed to parse YAML: document ended inside a collection");
                }
            }
            Event::Alias(anchor) => {
                let anchor = anchors
                    .get(&anchor)
                    .ok_or_else(|| located_error(span, "undefined or cyclic YAML alias"))?;
                budget.alias(anchor, frames.len(), span)?;
                attach_node(anchor.node.clone(), &mut frames, &mut root)?;
            }
            Event::Scalar(value, style, anchor, tag) => {
                ensure_depth(frames.len() + 1, span)?;
                budget.nodes(1, span)?;
                let node = YamlNode::Scalar {
                    value: value.into_owned(),
                    style,
                    tag: tag.map(|tag| tag.into_owned()),
                    span,
                };
                if anchor != 0 {
                    anchors.insert(
                        anchor,
                        Anchor {
                            node: node.clone(),
                            nodes: 1,
                            depth: 1,
                        },
                    );
                }
                attach_node(node, &mut frames, &mut root)?;
            }
            Event::SequenceStart(_, anchor, tag) => {
                ensure_depth(frames.len() + 1, span)?;
                frames.push(Frame::Sequence {
                    values: Vec::new(),
                    anchor,
                    tag: tag.map(|tag| tag.into_owned()),
                    span,
                });
            }
            Event::SequenceEnd => {
                let Some(Frame::Sequence {
                    values,
                    anchor,
                    tag,
                    span,
                }) = frames.pop()
                else {
                    bail!("Failed to parse YAML: unexpected sequence end");
                };
                let node = YamlNode::Sequence { values, tag, span };
                budget.nodes(1, span)?;
                if anchor != 0 {
                    let (nodes, depth) = node_metrics(&node);
                    anchors.insert(
                        anchor,
                        Anchor {
                            node: node.clone(),
                            nodes,
                            depth,
                        },
                    );
                }
                attach_node(node, &mut frames, &mut root)?;
            }
            Event::MappingStart(_, anchor, tag) => {
                ensure_depth(frames.len() + 1, span)?;
                frames.push(Frame::Mapping {
                    entries: Vec::new(),
                    pending_key: None,
                    anchor,
                    tag: tag.map(|tag| tag.into_owned()),
                    span,
                });
            }
            Event::MappingEnd => {
                let Some(Frame::Mapping {
                    entries,
                    pending_key,
                    anchor,
                    tag,
                    span,
                }) = frames.pop()
                else {
                    bail!("Failed to parse YAML: unexpected mapping end");
                };
                if pending_key.is_some() {
                    bail!("Failed to parse YAML: mapping key has no value");
                }
                let node = YamlNode::Mapping { entries, tag, span };
                budget.nodes(1, span)?;
                if anchor != 0 {
                    let (nodes, depth) = node_metrics(&node);
                    anchors.insert(
                        anchor,
                        Anchor {
                            node: node.clone(),
                            nodes,
                            depth,
                        },
                    );
                }
                attach_node(node, &mut frames, &mut root)?;
            }
            _ => bail!("Failed to parse YAML: unsupported parser event"),
        }
    }

    if documents != 1 {
        bail!("Failed to parse YAML: expected exactly one document");
    }
    root.ok_or_else(|| anyhow!("Failed to parse YAML: document has no value"))
}

fn attach_node(node: YamlNode, frames: &mut [Frame], root: &mut Option<YamlNode>) -> Result<()> {
    match frames.last_mut() {
        Some(Frame::Sequence { values, .. }) => values.push(node),
        Some(Frame::Mapping {
            entries,
            pending_key,
            ..
        }) => {
            if let Some(key) = pending_key.take() {
                entries.push((*key, node));
            } else {
                *pending_key = Some(Box::new(node));
            }
        }
        None if root.is_none() => *root = Some(node),
        None => bail!("Failed to parse YAML: document contains more than one root value"),
    }
    Ok(())
}

fn ensure_depth(depth: usize, span: Span) -> Result<()> {
    if depth > MAX_YAML_DEPTH {
        Err(located_error(span, "YAML nesting depth limit exceeded"))
    } else {
        Ok(())
    }
}

fn node_metrics(node: &YamlNode) -> (usize, usize) {
    match node {
        YamlNode::Scalar { .. } => (1, 1),
        YamlNode::Sequence { values, .. } => {
            let mut nodes = 1;
            let mut depth = 1;
            for value in values {
                let (child_nodes, child_depth) = node_metrics(value);
                nodes += child_nodes;
                depth = depth.max(child_depth + 1);
            }
            (nodes, depth)
        }
        YamlNode::Mapping { entries, .. } => {
            let mut nodes = 1;
            let mut depth = 1;
            for (key, value) in entries {
                for child in [key, value] {
                    let (child_nodes, child_depth) = node_metrics(child);
                    nodes += child_nodes;
                    depth = depth.max(child_depth + 1);
                }
            }
            (nodes, depth)
        }
    }
}

fn node_to_json(
    node: &YamlNode,
    path: &str,
    mapping_key: bool,
    depth: usize,
) -> Result<serde_json::Value> {
    ensure_depth(depth, node_span(node))?;
    match node {
        YamlNode::Scalar {
            value,
            style,
            tag,
            span,
        } => scalar_to_json(value, *style, tag.as_ref(), *span, path, mapping_key),
        YamlNode::Sequence { values, tag, span } => {
            validate_collection_tag(tag.as_ref(), "seq", *span)?;
            if mapping_key {
                return Err(located_error(*span, "YAML mapping keys must be strings"));
            }
            values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    node_to_json(value, &format!("{path}[{index}]"), false, depth + 1)
                })
                .collect::<Result<Vec<_>>>()
                .map(serde_json::Value::Array)
        }
        YamlNode::Mapping { entries, tag, span } => {
            validate_collection_tag(tag.as_ref(), "map", *span)?;
            if mapping_key {
                return Err(located_error(*span, "YAML mapping keys must be strings"));
            }
            mapping_to_json(entries, path, depth)
        }
    }
}

fn mapping_to_json(
    entries: &[(YamlNode, YamlNode)],
    path: &str,
    depth: usize,
) -> Result<serde_json::Value> {
    let mut merged = BTreeMap::<String, serde_json::Value>::new();
    let mut explicit = BTreeMap::<String, serde_json::Value>::new();
    let mut saw_merge = false;

    for (key_node, value_node) in entries {
        if is_merge_key(key_node) {
            if saw_merge {
                return Err(node_error(key_node, "duplicate YAML mapping key \"<<\""));
            }
            saw_merge = true;
            for source in merge_sources(value_node, path, depth + 1)? {
                for (key, value) in source {
                    merged.entry(key).or_insert(value);
                }
            }
            continue;
        }

        let key_value = node_to_json(key_node, path, true, depth + 1)?;
        let serde_json::Value::String(key) = key_value else {
            return Err(node_error(
                key_node,
                "YAML mapping keys must resolve as strings",
            ));
        };
        if explicit.contains_key(&key) {
            return Err(node_error(
                key_node,
                &format!("duplicate YAML mapping key {key:?}"),
            ));
        }
        let child_path = json_path(path, &key);
        explicit.insert(
            key,
            node_to_json(value_node, &child_path, false, depth + 1)?,
        );
    }

    merged.extend(explicit);
    Ok(serde_json::Value::Object(merged.into_iter().collect()))
}

fn merge_sources(
    node: &YamlNode,
    path: &str,
    depth: usize,
) -> Result<Vec<BTreeMap<String, serde_json::Value>>> {
    let nodes: Vec<(&YamlNode, usize)> = match node {
        YamlNode::Mapping { .. } => vec![(node, depth)],
        YamlNode::Sequence {
            values, tag, span, ..
        } => {
            validate_collection_tag(tag.as_ref(), "seq", *span)?;
            values.iter().map(|value| (value, depth + 1)).collect()
        }
        _ => {
            return Err(node_error(
                node,
                "YAML merge value must be a mapping or sequence of mappings",
            ));
        }
    };

    nodes
        .into_iter()
        .map(
            |(node, depth)| match node_to_json(node, path, false, depth)? {
                serde_json::Value::Object(map) => Ok(map.into_iter().collect()),
                _ => Err(node_error(
                    node,
                    "YAML merge sequence entries must be mappings",
                )),
            },
        )
        .collect()
}

fn is_merge_key(node: &YamlNode) -> bool {
    matches!(
        node,
        YamlNode::Scalar {
            value,
            style: ScalarStyle::Plain,
            tag: None,
            ..
        } if value == "<<"
    )
}

fn scalar_to_json(
    value: &str,
    style: ScalarStyle,
    tag: Option<&Tag>,
    span: Span,
    path: &str,
    mapping_key: bool,
) -> Result<serde_json::Value> {
    let core_tag = match tag {
        Some(tag) => Some(tag.core_suffix().ok_or_else(|| {
            located_error(span, &format!("unsupported YAML tag {}", tag.original()))
        })?),
        None => None,
    };

    let parsed = match core_tag {
        Some("str") => serde_json::Value::String(value.to_string()),
        Some("null") => parse_null(value, span)?,
        Some("bool") => parse_bool(value, span)?,
        Some("int") => parse_integer(value, span)?,
        Some("float") => parse_float(value, span)?,
        Some(other) => {
            return Err(located_error(
                span,
                &format!("YAML tag !!{other} cannot be applied to a scalar"),
            ));
        }
        None if style != ScalarStyle::Plain => serde_json::Value::String(value.to_string()),
        None => parse_plain_scalar(value, span)?,
    };

    if mapping_key && !parsed.is_string() {
        return Err(located_error(
            span,
            &format!("YAML mapping key at {path} must resolve as a string"),
        ));
    }
    Ok(parsed)
}

fn parse_plain_scalar(value: &str, span: Span) -> Result<serde_json::Value> {
    if is_null(value) {
        return Ok(serde_json::Value::Null);
    }
    if is_bool(value) {
        return parse_bool(value, span);
    }
    if looks_like_integer(value) {
        return parse_integer(value, span);
    }
    if looks_like_float(value) {
        return parse_float(value, span);
    }
    Ok(serde_json::Value::String(value.to_string()))
}

fn parse_null(value: &str, span: Span) -> Result<serde_json::Value> {
    if is_null(value) {
        Ok(serde_json::Value::Null)
    } else {
        Err(located_error(span, "invalid YAML null scalar"))
    }
}

fn is_null(value: &str) -> bool {
    matches!(value, "" | "~" | "null" | "Null" | "NULL")
}

fn parse_bool(value: &str, span: Span) -> Result<serde_json::Value> {
    match value {
        "true" | "True" | "TRUE" => Ok(serde_json::Value::Bool(true)),
        "false" | "False" | "FALSE" => Ok(serde_json::Value::Bool(false)),
        _ => Err(located_error(span, "invalid YAML 1.2 boolean scalar")),
    }
}

fn is_bool(value: &str) -> bool {
    matches!(
        value,
        "true" | "True" | "TRUE" | "false" | "False" | "FALSE"
    )
}

fn looks_like_integer(value: &str) -> bool {
    if let Some(rest) = value.strip_prefix("0o") {
        return !rest.is_empty() && rest.bytes().all(|b| matches!(b, b'0'..=b'7'));
    }
    if let Some(rest) = value.strip_prefix("0x") {
        return !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_hexdigit());
    }
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    if unsigned.is_empty() {
        return false;
    }
    unsigned.bytes().all(|b| b.is_ascii_digit())
}

fn parse_integer(value: &str, span: Span) -> Result<serde_json::Value> {
    if !looks_like_integer(value) {
        return Err(located_error(span, "invalid YAML integer scalar"));
    }
    let (negative, radix, digits) = if let Some(rest) = value.strip_prefix('-') {
        (true, 10, rest)
    } else if let Some(rest) = value.strip_prefix('+') {
        (false, 10, rest)
    } else if let Some(rest) = value.strip_prefix("0o") {
        (false, 8, rest)
    } else if let Some(rest) = value.strip_prefix("0x") {
        (false, 16, rest)
    } else {
        (false, 10, value)
    };

    let magnitude = u128::from_str_radix(digits, radix)
        .map_err(|_| located_error(span, "invalid or out-of-range YAML integer"))?;
    let number = if negative {
        let max_magnitude = i64::MAX as u128 + 1;
        if magnitude > max_magnitude {
            return Err(located_error(
                span,
                "YAML integer is outside the JSON range",
            ));
        }
        serde_json::Number::from(if magnitude == max_magnitude {
            i64::MIN
        } else {
            -(magnitude as i64)
        })
    } else {
        serde_json::Number::from(
            u64::try_from(magnitude)
                .map_err(|_| located_error(span, "YAML integer is outside the JSON range"))?,
        )
    };
    Ok(serde_json::Value::Number(number))
}

fn looks_like_float(value: &str) -> bool {
    if parse_special_float(value).is_some() {
        return true;
    }
    if value.contains('_') {
        return false;
    }
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    (unsigned.contains('.') || unsigned.contains('e') || unsigned.contains('E'))
        && unsigned.bytes().any(|b| b.is_ascii_digit())
        && value.parse::<f64>().is_ok()
}

fn parse_float(value: &str, span: Span) -> Result<serde_json::Value> {
    if value.contains('_') {
        return Err(located_error(span, "invalid YAML float scalar"));
    }
    let parsed = match parse_special_float(value) {
        Some(value) => value,
        None => value
            .parse::<f64>()
            .map_err(|_| located_error(span, "invalid YAML float scalar"))?,
    };
    let number = serde_json::Number::from_f64(parsed)
        .ok_or_else(|| located_error(span, "non-finite YAML floats are not supported"))?;
    Ok(serde_json::Value::Number(number))
}

fn parse_special_float(value: &str) -> Option<f64> {
    match value {
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => Some(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => Some(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => Some(f64::NAN),
        _ => None,
    }
}

fn validate_collection_tag(tag: Option<&Tag>, expected: &str, span: Span) -> Result<()> {
    let Some(tag) = tag else {
        return Ok(());
    };
    if tag.is_yaml_core_schema_tag(expected) {
        Ok(())
    } else if tag.is_yaml_core_schema() {
        Err(located_error(
            span,
            &format!(
                "YAML tag {} cannot be applied to this collection",
                tag.original()
            ),
        ))
    } else {
        Err(located_error(
            span,
            &format!("unsupported YAML tag {}", tag.original()),
        ))
    }
}

fn node_span(node: &YamlNode) -> Span {
    match node {
        YamlNode::Scalar { span, .. }
        | YamlNode::Sequence { span, .. }
        | YamlNode::Mapping { span, .. } => *span,
    }
}

fn node_error(node: &YamlNode, message: &str) -> anyhow::Error {
    located_error(node_span(node), message)
}

fn located_error(span: Span, message: &str) -> anyhow::Error {
    let marker = span.tag_start().unwrap_or(span.start);
    anyhow!(
        "Failed to parse YAML at line {}, column {}: {message}",
        marker.line(),
        marker.col() + 1
    )
}

fn json_path(parent: &str, key: &str) -> String {
    if key
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        format!("{parent}.{key}")
    } else {
        format!("{parent}[{key:?}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn renders_deterministic_yaml() {
        let value = json!({ "z": [1, 2], "a": { "enabled": true } });
        assert_eq!(
            YamlRenderer.render(&value).unwrap(),
            "a:\n  enabled: true\nz:\n  - 1\n  - 2\n"
        );
    }

    #[test]
    fn roundtrips_supported_json_values() {
        let values = [
            serde_json::Value::Null,
            json!(true),
            json!(42),
            json!(1.5),
            json!("yes"),
            json!([
                null,
                false,
                "001",
                "0b10",
                "tRuE",
                "nUlL",
                "-0xF",
                ".iNf",
                { "nested": "value" }
            ]),
        ];
        for original in values {
            let rendered = YamlRenderer.render(&original).unwrap();
            let parsed = YamlRenderer.parse(&rendered).unwrap();
            assert_eq!(parsed, original, "rendered YAML:\n{rendered}");
        }
    }

    #[test]
    fn parses_yaml_1_2_scalars() {
        let parsed = YamlRenderer
            .parse("yes: off\nnull_title: Null\nnull_upper: NULL\ntrue_title: True\ntrue_upper: TRUE\nfalse_title: False\nfalse_upper: FALSE\nnumber: 0247\noctal: 0o247\nhex: 0xF\nfraction: .5\ntime: 190:20:30\n")
            .unwrap();
        assert_eq!(parsed["yes"], "off");
        assert_eq!(parsed["null_title"], serde_json::Value::Null);
        assert_eq!(parsed["null_upper"], serde_json::Value::Null);
        assert_eq!(parsed["true_title"], true);
        assert_eq!(parsed["true_upper"], true);
        assert_eq!(parsed["false_title"], false);
        assert_eq!(parsed["false_upper"], false);
        assert_eq!(parsed["number"], 247);
        assert_eq!(parsed["octal"], 167);
        assert_eq!(parsed["hex"], 15);
        assert_eq!(parsed["fraction"], 0.5);
        assert_eq!(parsed["time"], "190:20:30");
    }

    #[test]
    fn non_core_scalar_spellings_remain_strings() {
        for scalar in ["0b10", "tRuE", "nUlL", "-0xF", "+0o7", ".iNf"] {
            let parsed = YamlRenderer.parse(&format!("value: {scalar}\n")).unwrap();
            assert_eq!(parsed["value"], scalar, "scalar {scalar:?}");
        }

        for tagged in [
            "!!int 0b10",
            "!!bool tRuE",
            "!!null nUlL",
            "!!int -0xF",
            "!!float .iNf",
        ] {
            assert!(
                YamlRenderer.parse(&format!("value: {tagged}\n")).is_err(),
                "tagged scalar {tagged:?}"
            );
        }
    }

    #[test]
    fn invalid_numeric_separators_do_not_become_numbers() {
        let parsed = YamlRenderer
            .parse("double: 1__2\ntrailing: 1_\nhex: 0x_1\nfloat: 1_.5\n")
            .unwrap();
        assert_eq!(parsed["double"], "1__2");
        assert_eq!(parsed["trailing"], "1_");
        assert_eq!(parsed["hex"], "0x_1");
        assert_eq!(parsed["float"], "1_.5");
        assert!(YamlRenderer.parse("value: !!int 1_2\n").is_err());
        assert!(YamlRenderer.parse("value: !!float 1_.5\n").is_err());
    }

    #[test]
    fn accepts_explicit_and_structurally_empty_documents_as_null() {
        assert_eq!(
            YamlRenderer.parse("---\n").unwrap(),
            serde_json::Value::Null
        );
        assert_eq!(
            YamlRenderer.parse("--- null\n...\n").unwrap(),
            serde_json::Value::Null
        );
    }

    #[test]
    fn rejects_empty_and_multiple_documents() {
        assert!(YamlRenderer.parse("").is_err());
        assert!(YamlRenderer.parse("a: 1\n---\nb: 2\n").is_err());
    }

    #[test]
    fn rejects_duplicate_and_non_string_keys() {
        assert!(YamlRenderer.parse("key: 1\nkey: 2\n").is_err());
        assert!(YamlRenderer.parse("<<: {one: 1}\n<<: {two: 2}\n").is_err());
        assert!(YamlRenderer.parse("true: value\n").is_err());
        assert!(YamlRenderer.parse("123: value\n").is_err());
        assert!(YamlRenderer.parse("[one, two]: value\n").is_err());
        let parsed = YamlRenderer
            .parse("\"true\": bool-like\n\"123\": numeric-like\n")
            .unwrap();
        assert_eq!(parsed["true"], "bool-like");
        assert_eq!(parsed["123"], "numeric-like");
    }

    #[test]
    fn expands_aliases_and_merge_keys() {
        let parsed = YamlRenderer
            .parse("defaults: &defaults\n  retries: 3\n  timeout: 30\nproduction:\n  <<: *defaults\n  timeout: 60\n")
            .unwrap();
        assert_eq!(parsed["production"], json!({ "retries": 3, "timeout": 60 }));
    }

    #[test]
    fn merge_sequences_use_first_source_precedence() {
        let parsed = YamlRenderer
            .parse("first: &first {shared: first, a: 1}\nsecond: &second {shared: second, b: 2}\ncombined:\n  <<: [*first, *second]\n")
            .unwrap();
        assert_eq!(
            parsed["combined"],
            json!({ "a": 1, "b": 2, "shared": "first" })
        );
    }

    #[test]
    fn merge_sequences_validate_their_tags() {
        assert!(
            YamlRenderer
                .parse("base: &base {value: 1}\nmerged:\n  <<: !Custom [*base]\n")
                .is_err()
        );
        assert!(
            YamlRenderer
                .parse("base: &base {value: 1}\nmerged:\n  <<: !!seq [*base]\n")
                .is_ok()
        );
    }

    #[test]
    fn rejects_non_finite_numbers_and_unsupported_tags() {
        assert!(YamlRenderer.parse("value: .inf\n").is_err());
        assert!(YamlRenderer.parse("value: +.INF\n").is_err());
        assert!(YamlRenderer.parse("value: -.inf\n").is_err());
        assert!(YamlRenderer.parse("value: .nan\n").is_err());
        assert!(YamlRenderer.parse("value: .NaN\n").is_err());
        assert!(YamlRenderer.parse("value: !Custom text\n").is_err());
        assert!(
            YamlRenderer
                .parse("value: !!timestamp 2026-08-27\n")
                .is_err()
        );
        assert!(YamlRenderer.parse("value: !!binary SGVsbG8=\n").is_err());
    }

    #[test]
    fn rejects_out_of_range_integers() {
        assert!(YamlRenderer.parse("value: 18446744073709551616\n").is_err());
        assert!(YamlRenderer.parse("value: -9223372036854775809\n").is_err());
    }

    #[test]
    fn enforces_parser_resource_budgets() {
        let event_heavy = "- 0\n".repeat(MAX_YAML_EVENTS);
        let event_error = YamlRenderer.parse(&event_heavy).unwrap_err().to_string();
        assert!(event_error.contains("event limit"), "{event_error}");

        let deeply_nested = format!(
            "value: {}0{}\n",
            "[".repeat(MAX_YAML_DEPTH),
            "]".repeat(MAX_YAML_DEPTH)
        );
        let depth_error = YamlRenderer.parse(&deeply_nested).unwrap_err().to_string();
        assert!(depth_error.contains("depth limit"), "{depth_error}");

        let aliases = vec!["*base"; MAX_YAML_ALIASES + 1].join(", ");
        let alias_heavy = format!("base: &base {{value: 1}}\nvalues: [{aliases}]\n");
        let alias_error = YamlRenderer.parse(&alias_heavy).unwrap_err().to_string();
        assert!(alias_error.contains("alias limit"), "{alias_error}");

        let mut expanding = "a0: &a0 [0]\n".to_string();
        for index in 1..20 {
            expanding.push_str(&format!(
                "a{index}: &a{index} [*a{}, *a{}]\n",
                index - 1,
                index - 1
            ));
        }
        let node_error = YamlRenderer.parse(&expanding).unwrap_err().to_string();
        assert!(node_error.contains("node limit"), "{node_error}");
    }

    #[test]
    fn preserves_multiline_strings_semantically() {
        let parsed = YamlRenderer
            .parse("message: |-\n  first line\n  second line\n")
            .unwrap();
        assert_eq!(parsed["message"], "first line\nsecond line");
    }
}
