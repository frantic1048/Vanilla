use colored::Colorize;
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
                eprintln!("warning: invalid ignore regex '{}': {}", p, e);
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

    for change in diff.iter_all_changes() {
        let line = change.value().trim_end();
        match change.tag() {
            ChangeTag::Delete => {
                output.push(format!("{} {}", "-".red(), line.red()));
            }
            ChangeTag::Insert => {
                output.push(format!("{} {}", "+".green(), line.green()));
            }
            ChangeTag::Equal => {
                output.push(format!("  {}", line.dimmed()));
            }
        }
    }

    DiffResult::with_changes(output.join("\n"))
}

/// Create a compact diff showing only changed lines with context
#[allow(dead_code)]
pub fn compact_diff(generated: &str, deployed: &str, context_lines: usize) -> DiffResult {
    if generated == deployed {
        return DiffResult::no_changes();
    }

    let diff = TextDiff::from_lines(deployed, generated);
    let mut output = Vec::new();

    for hunk in diff.unified_diff().context_radius(context_lines).iter_hunks() {
        output.push(format!("{}", hunk.header().to_string().dimmed()));
        for change in hunk.iter_changes() {
            let line = change.value().trim_end();
            match change.tag() {
                ChangeTag::Delete => {
                    output.push(format!("{}{}", "-".red(), line.red()));
                }
                ChangeTag::Insert => {
                    output.push(format!("{}{}", "+".green(), line.green()));
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
        assert!(result.output.contains("old"));
        assert!(result.output.contains("new"));
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
    }
}
