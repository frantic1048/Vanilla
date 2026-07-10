use std::borrow::Cow;
use std::fmt;

/// A structured key path: the exact key segments leading to a config value.
///
/// Segments are real JSON/Nickel keys and may themselves contain dots or
/// brackets (e.g. VS Code's `"[javascript]"` or `"editor.codeActionsOnSave"`).
/// Joining with `.` is therefore display-only and must never be parsed back
/// into segments.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct KeyPath(Vec<String>);

impl KeyPath {
    pub fn new(segments: Vec<String>) -> Self {
        KeyPath(segments)
    }

    /// The empty path (points at the root value).
    pub fn root() -> Self {
        KeyPath(Vec::new())
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    pub fn segments(&self) -> &[String] {
        &self.0
    }

    /// A new path with one more segment appended.
    pub fn child(&self, segment: impl Into<String>) -> Self {
        let mut segments = self.0.clone();
        segments.push(segment.into());
        KeyPath(segments)
    }

    /// The path without its last segment; `None` for the root path.
    pub fn parent(&self) -> Option<KeyPath> {
        if self.0.is_empty() {
            None
        } else {
            Some(KeyPath(self.0[..self.0.len() - 1].to_vec()))
        }
    }

    pub fn last(&self) -> Option<&str> {
        self.0.last().map(String::as_str)
    }
}

impl fmt::Display for KeyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, segment) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            f.write_str(&display_segment(segment))?;
        }
        Ok(())
    }
}

/// Render one path segment for display. Segments containing the `.` join
/// character are quoted Nickel-style so `"a.b"` (one literal key) and `a.b`
/// (nested `a` then `b`) stay distinguishable in joined output.
pub fn display_segment(segment: &str) -> Cow<'_, str> {
    if segment.is_empty() || segment.contains('.') || segment.contains('"') {
        Cow::Owned(format!(
            "\"{}\"",
            segment.replace('\\', "\\\\").replace('"', "\\\"")
        ))
    } else {
        Cow::Borrowed(segment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_path_basics() {
        let p = KeyPath::root()
            .child("[javascript]")
            .child("editor.codeActionsOnSave");
        assert_eq!(p.segments().len(), 2);
        assert_eq!(p.segments(), ["[javascript]", "editor.codeActionsOnSave"]);
        assert_eq!(p.to_string(), "[javascript].\"editor.codeActionsOnSave\"");
        assert_eq!(p.parent().unwrap().segments(), ["[javascript]"]);
        assert_eq!(p.last(), Some("editor.codeActionsOnSave"));
        assert!(KeyPath::root().parent().is_none());
    }

    #[test]
    fn test_display_disambiguates_literal_dot_from_nested() {
        let literal = KeyPath::new(vec!["a.b".to_string()]);
        let nested = KeyPath::root().child("a").child("b");
        assert_eq!(literal.to_string(), "\"a.b\"");
        assert_eq!(nested.to_string(), "a.b");
        assert_ne!(literal.to_string(), nested.to_string());
    }

    #[test]
    fn test_display_segment_quoting() {
        assert_eq!(display_segment("plain"), "plain");
        assert_eq!(display_segment("[javascript]"), "[javascript]");
        assert_eq!(display_segment("a.b"), "\"a.b\"");
        assert_eq!(display_segment(""), "\"\"");
        assert_eq!(display_segment("say \"hi\""), "\"say \\\"hi\\\"\"");
    }
}
