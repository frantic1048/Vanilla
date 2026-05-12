use console::style;
use regex::Regex;
use similar::{ChangeTag, TextDiff};

use super::DiffResult;

/// Filter out lines matching any of the ignore patterns (regex)
fn filter_ignored_lines(content: &str, ignore_patterns: &[Regex]) -> String {
    if ignore_patterns.is_empty() {
        return content.to_string();
    }
    content
        .lines()
        .filter(|line| !ignore_patterns.iter().any(|re| re.is_match(line)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Perform unified text diff with optional regex-based line filtering
pub fn text_diff(generated: &str, deployed: &str, ignore_patterns: &[String]) -> DiffResult {
    let regexes: Vec<Regex> = ignore_patterns
        .iter()
        .filter_map(|p| match Regex::new(p) {
            Ok(re) => Some(re),
            Err(e) => {
                eprintln!("warning: invalid ignore regex '{p}': {e}");
                None
            }
        })
        .collect();

    let gen_filtered = filter_ignored_lines(generated, &regexes);
    let dep_filtered = filter_ignored_lines(deployed, &regexes);

    if gen_filtered == dep_filtered {
        return DiffResult::no_changes();
    }

    let diff = TextDiff::from_lines(&dep_filtered, &gen_filtered);
    let mut output = Vec::new();
    let mut source_lines = Vec::new();
    let mut target_lines = Vec::new();

    for change in diff.iter_all_changes() {
        let line = change.value().trim_end();
        match change.tag() {
            ChangeTag::Delete => {
                target_lines.push(line.to_string());
            }
            ChangeTag::Insert => {
                source_lines.push(line.to_string());
            }
            ChangeTag::Equal => {
                flush_text_side_lines(&mut output, &mut source_lines, &mut target_lines, None);
                output.push(format!("  {}", style(line).dim()));
            }
        }
    }
    flush_text_side_lines(&mut output, &mut source_lines, &mut target_lines, None);

    DiffResult::with_changes(output.join("\n"))
}

/// Perform text diff and include an approximate same-line Base value when a
/// snapshot is available. This keeps the prompt compact while giving users the
/// common ancestor next to Source/Target lines.
pub fn text_diff_with_base(
    generated: &str,
    deployed: &str,
    base: &str,
    ignore_patterns: &[String],
) -> DiffResult {
    let regexes: Vec<Regex> = ignore_patterns
        .iter()
        .filter_map(|p| match Regex::new(p) {
            Ok(re) => Some(re),
            Err(e) => {
                eprintln!("warning: invalid ignore regex '{p}': {e}");
                None
            }
        })
        .collect();

    let gen_filtered = filter_ignored_lines(generated, &regexes);
    let dep_filtered = filter_ignored_lines(deployed, &regexes);
    let base_filtered = filter_ignored_lines(base, &regexes);

    if gen_filtered == dep_filtered {
        return DiffResult::no_changes();
    }

    let base_lines: Vec<&str> = base_filtered.lines().collect();
    let diff = TextDiff::from_lines(&dep_filtered, &gen_filtered);
    let mut output = Vec::new();
    let mut source_lines = Vec::new();
    let mut target_lines = Vec::new();
    let mut base_side_lines = Vec::new();

    for change in diff.iter_all_changes() {
        let line = change.value().trim_end();
        match change.tag() {
            ChangeTag::Delete => {
                target_lines.push(line.to_string());
                push_base_line(
                    &mut base_side_lines,
                    base_line_at(&base_lines, change.old_index()),
                );
            }
            ChangeTag::Insert => {
                source_lines.push(line.to_string());
                push_base_line(
                    &mut base_side_lines,
                    base_line_at(&base_lines, change.new_index()),
                );
            }
            ChangeTag::Equal => {
                flush_text_side_lines(
                    &mut output,
                    &mut source_lines,
                    &mut target_lines,
                    Some(&mut base_side_lines),
                );
                output.push(format!("  {}", style(line).dim()));
            }
        }
    }
    flush_text_side_lines(
        &mut output,
        &mut source_lines,
        &mut target_lines,
        Some(&mut base_side_lines),
    );

    DiffResult::with_changes(output.join("\n"))
}

fn flush_text_side_lines(
    output: &mut Vec<String>,
    source_lines: &mut Vec<String>,
    target_lines: &mut Vec<String>,
    base_lines: Option<&mut Vec<String>>,
) {
    for line in source_lines.drain(..) {
        output.push(format!("{}", style(format!("<< Source: {line}")).blue()));
    }
    for line in target_lines.drain(..) {
        output.push(format!("{}", style(format!(">> Target: {line}")).magenta()));
    }
    if let Some(base_lines) = base_lines {
        for line in base_lines.drain(..) {
            output.push(format!("{}", style(format!("|| Base: {line}")).color256(8)));
        }
    }
}

fn push_base_line(base_lines: &mut Vec<String>, line: &str) {
    if base_lines.last().is_some_and(|last| last == line) {
        return;
    }
    base_lines.push(line.to_string());
}

fn base_line_at<'a>(base_lines: &'a [&str], index: Option<usize>) -> &'a str {
    index
        .and_then(|i| base_lines.get(i).copied())
        .unwrap_or("<missing>")
        .trim_end()
}

/// Create a compact diff showing only changed lines with context
#[allow(dead_code)]
pub fn compact_diff(generated: &str, deployed: &str, context_lines: usize) -> DiffResult {
    if generated == deployed {
        return DiffResult::no_changes();
    }

    let diff = TextDiff::from_lines(deployed, generated);
    let mut output = Vec::new();

    for hunk in diff
        .unified_diff()
        .context_radius(context_lines)
        .iter_hunks()
    {
        output.push(format!("{}", style(hunk.header().to_string()).dim()));
        for change in hunk.iter_changes() {
            let line = change.value().trim_end();
            match change.tag() {
                ChangeTag::Delete => {
                    output.push(format!("{}", style(format!(">> Target: {line}")).magenta()));
                }
                ChangeTag::Insert => {
                    output.push(format!("{}", style(format!("<< Source: {line}")).blue()));
                }
                ChangeTag::Equal => {
                    output.push(format!(" {}", line));
                }
            }
        }
    }

    if output.is_empty() {
        DiffResult::no_changes()
    } else {
        DiffResult::with_changes(output.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_diff_no_changes() {
        let result = text_diff("hello\nworld", "hello\nworld", &[]);
        assert!(!result.has_changes);
    }

    #[test]
    fn test_text_diff_with_changes() {
        let result = text_diff("hello\nnew", "hello\nold", &[]);
        assert!(result.has_changes);
        assert!(result.output.contains(">> Target: old"));
        assert!(result.output.contains("<< Source: new"));
        assert!(result.output.contains("old"));
        assert!(result.output.contains("new"));
    }

    #[test]
    fn test_text_diff_with_base_shows_base_side() {
        let result = text_diff_with_base("hello\nnew", "hello\nold", "hello\noriginal", &[]);
        assert!(result.has_changes);
        assert!(result.output.contains(">> Target: old"));
        assert!(result.output.contains("<< Source: new"));
        assert!(result.output.contains("|| Base: original"));
    }

    #[test]
    fn test_text_diff_with_ignore() {
        let generated = "key1=a\nkey2=b";
        let deployed = "key1=a\nkey2=b\ntree_view=1\ncolor_scheme=6";
        let ignore = vec!["^tree_view".to_string(), "^color_scheme".to_string()];
        let result = text_diff(generated, deployed, &ignore);
        assert!(!result.has_changes);
    }

    #[test]
    fn test_compact_diff() {
        let old = "line1\nline2\nline3\nline4\nline5";
        let new = "line1\nmodified\nline3\nline4\nline5";
        let result = compact_diff(new, old, 1);
        assert!(result.has_changes);
        assert!(result.output.contains(">> Target: line2"));
        assert!(result.output.contains("<< Source: modified"));
    }
}
