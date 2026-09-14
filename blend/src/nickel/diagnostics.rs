use std::path::Path;

use codespan_reporting::{
    diagnostic::Severity,
    term::{self, termcolor::NoColor},
};
use nickel_lang_core::{
    error::{Error, EvalErrorKind, IntoDiagnostics},
    files::{FileId, Files},
};

use super::source_map::SourceMap;

/// Render Nickel's source excerpts against the original order. Imports retain
/// their own files and locations; generated-only spans are explicitly named.
pub(super) fn render_error(
    error: Error,
    mut files: Files,
    file_id: FileId,
    source: &SourceMap,
    path: &Path,
    operation: &str,
    resolver_entry: Option<usize>,
) -> anyhow::Error {
    let mut message = format!("Nickel {operation} error in {}", path.display());
    if let Some(index) = resolver_entry {
        message.push_str(&format!(" (from_config entry {index})"));
    }
    message.push_str(":\n");

    let unmatched_pattern = matches!(
        &error,
        Error::EvalError(error) if matches!(
            &error.error,
            EvalErrorKind::NonExhaustiveMatch { .. } | EvalErrorKind::NonExhaustiveEnumMatch { .. }
        )
    );
    let diagnostics = error.into_diagnostics(&mut files);
    let original_id = files.add(path.as_os_str(), source.original.clone());
    let generated_id = files.add("<Blend generated>", source.generated.clone());
    let mut buffer = NoColor::new(Vec::new());
    for mut diagnostic in diagnostics {
        // Nickel emits call-stack frames as notes with empty titles.
        // Named notes (e.g. parent contract violations) remain useful.
        if diagnostic.message.is_empty() {
            continue;
        }
        for label in &mut diagnostic.labels {
            if label.file_id == file_id {
                if let Some(range) = source.original_range(&label.range) {
                    label.file_id = original_id;
                    label.range = range;
                } else {
                    label.file_id = generated_id;
                }
            }
        }

        if resolver_entry.is_some() && unmatched_pattern && diagnostic.severity == Severity::Error {
            diagnostic.notes.push(
                "A match in the resolver made no decision; add an explicit fallback \
                 branch such as `_ => 'Unresolved`."
                    .to_owned(),
            );
        }
        if let Err(error) =
            term::emit_to_write_style(&mut buffer, &term::Config::default(), &files, &diagnostic)
        {
            // Retain the cause even if a future upstream span cannot be rendered.
            message.push_str(&format!(
                "{}\nCould not render source excerpt: {error}\n",
                diagnostic.message
            ));
        }
    }
    message.push_str(String::from_utf8_lossy(&buffer.into_inner()).trim_end());
    anyhow::anyhow!(message)
}

#[cfg(test)]
mod tests {
    use super::super::loader::eval_to_json;
    use super::*;

    fn evaluate_error(source: &str, resolver_entry: Option<usize>) -> String {
        let mut mapped = SourceMap::new(source);
        mapped.push_original(0..source.len());
        eval_to_json(&mapped, Path::new("order.ncl"), resolver_entry)
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn merge_conflict_explains_incompatible_types() {
        let message = evaluate_error("{ enabled = 'Unmanaged, enabled = false }", None);
        assert!(message.contains("non mergeable terms"), "{message}");
        assert!(
            message.contains("Bool") && message.contains("EnumTag"),
            "{message}"
        );
        assert!(
            message.contains("Values of different types can't be merged"),
            "{message}"
        );
        assert!(
            !message.contains("EvalError") && !message.contains("PosIdx"),
            "{message}"
        );
        assert!(
            message.contains("'Unmanaged") && message.contains("false"),
            "{message}"
        );
        assert!(message.contains("order.ncl:1:"), "{message}");
        assert!(message.lines().count() < 25, "{message}");
    }

    #[test]
    fn syntax_error_is_readable() {
        let message = evaluate_error("{ enabled = }", None);
        assert!(message.contains("unexpected token"), "{message}");
        assert!(!message.contains("ParseError"), "{message}");
    }

    #[test]
    fn export_error_is_readable() {
        let message = evaluate_error("{ value = fun x => x }", None);
        assert!(message.contains("JSON export error"), "{message}");
        assert!(message.contains("non serializable term"), "{message}");
        assert!(!message.contains("ExportError"), "{message}");
    }

    #[test]
    fn partial_matches_keep_fallback_advice() {
        // General patterns and enum-only matches use different Nickel errors.
        // Nested failures must remain errors too, never become 'Unresolved.
        for source in [
            "2 |> match { 1 => true }",
            "'Other |> match { 'Known => true }",
            "(fun _ => 2 |> match { 1 => true }) null",
        ] {
            let message = evaluate_error(source, Some(3));
            assert!(message.contains("from_config entry 3"), "{message}");
            assert!(message.contains("unmatched pattern"), "{message}");
            assert!(message.contains("resolver made no decision"), "{message}");
            assert!(message.contains("_ => 'Unresolved"), "{message}");
        }
    }

    #[test]
    fn user_error_text_does_not_trigger_partial_match_advice() {
        let message = evaluate_error(
            r#"{}."NonExhaustiveMatch NonExhaustiveEnumMatch unmatched pattern""#,
            Some(0),
        );
        assert!(message.contains("NonExhaustiveMatch"), "{message}");
        assert!(!message.contains("_ => 'Unresolved"), "{message}");
    }

    #[test]
    fn imported_conflicts_keep_both_source_files() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("order.ncl");
        let imported = temp.path().join("settings.ncl");
        std::fs::write(&imported, "{\n  enabled = false,\n}").unwrap();
        let source = "{ enabled = 'Unmanaged } & (import \"./settings.ncl\")";
        let mut mapped = SourceMap::new(source);
        mapped.push_original(0..source.len());
        mapped.wrap("let wrapper = (\n", ") in wrapper");
        let message = eval_to_json(&mapped, &path, None).unwrap_err().to_string();
        assert!(
            message.contains(&format!("{}:1:", path.display())),
            "{message}"
        );
        assert!(
            message.contains(&format!("{}:2:", imported.display())),
            "{message}"
        );
        assert!(
            message.contains("'Unmanaged") && message.contains("enabled = false"),
            "{message}"
        );
        assert!(!message.contains("<Blend generated>"), "{message}");
    }

    #[test]
    fn generated_only_labels_are_named_explicitly() {
        let mut mapped = SourceMap::new("1");
        mapped.push_original(0..1);
        mapped.wrap("let wrapper = (", ") in wrapper + true");
        let message = eval_to_json(&mapped, Path::new("order.ncl"), None)
            .unwrap_err()
            .to_string();
        assert!(message.contains("<Blend generated>:1:"), "{message}");
        assert!(!message.contains("order.ncl:1:"), "{message}");
        assert!(message.contains("true"), "{message}");
    }
}
