use std::ops::Range;

/// An evaluated program and the original order fragments copied into it.
/// Generated helpers and injected metadata deliberately have no source mapping.
pub(super) struct SourceMap {
    pub original: String,
    pub generated: String,
    fragments: Vec<(Range<usize>, Range<usize>)>,
}

impl SourceMap {
    pub fn new(original: &str) -> Self {
        Self {
            original: original.to_owned(),
            generated: String::new(),
            fragments: Vec::new(),
        }
    }

    pub fn push_original(&mut self, range: Range<usize>) {
        let start = self.generated.len();
        self.generated.push_str(&self.original[range.clone()]);
        self.fragments.push((start..self.generated.len(), range));
    }

    pub fn push_generated(&mut self, text: &str) {
        self.generated.push_str(text);
    }

    pub fn wrap(&mut self, prefix: &str, suffix: &str) {
        for (generated, _) in &mut self.fragments {
            generated.start += prefix.len();
            generated.end += prefix.len();
        }
        self.generated = format!("{prefix}{}{suffix}", self.generated);
    }

    /// Map both boundaries, including spans crossing metadata insertions.
    /// Nickel can widen a source expression's span to include our surrounding
    /// parentheses. Trim those delimiters, but never generated expressions.
    pub fn original_range(&self, range: &Range<usize>) -> Option<Range<usize>> {
        let start = self.fragments.iter().find_map(|(generated, original)| {
            if generated.contains(&range.start)
                || (range.is_empty() && range.start == generated.end)
            {
                Some(original.start + range.start - generated.start)
            } else if range.start < generated.start && generated.start < range.end {
                self.generated
                    .get(range.start..generated.start)
                    .filter(|prefix| prefix.chars().all(|c| c.is_whitespace() || c == '('))
                    .map(|_| original.start)
            } else {
                None
            }
        })?;
        let end = self
            .fragments
            .iter()
            .rev()
            .find_map(|(generated, original)| {
                if generated.start <= range.end && range.end <= generated.end {
                    Some(original.start + range.end - generated.start)
                } else if range.start < generated.end && generated.end < range.end {
                    self.generated
                        .get(generated.end..range.end)
                        .filter(|suffix| suffix.chars().all(|c| c.is_whitespace() || c == ')'))
                        .map(|_| original.end)
                } else {
                    None
                }
            })?;
        (start <= end).then_some(start..end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_original_fragments_and_spans_across_insertions() {
        let mut source = SourceMap::new("éimport rest");
        source.push_original(0..2);
        source.push_generated("((");
        source.push_original(2..8);
        source.push_generated(") & { injected = true })");
        source.push_original(8..source.original.len());
        source.wrap("let wrapper = (", ") in wrapper");

        let start = source.generated.find('é').unwrap();
        assert_eq!(source.original_range(&(start..start + 2)), Some(0..2));
        let import = source.generated.find("import").unwrap();
        assert_eq!(source.original_range(&(import..import + 6)), Some(2..8));
        let end = source.generated.find("rest").unwrap() + 4;
        assert_eq!(source.original_range(&(start..end)), Some(0..13));
        assert_eq!(source.original_range(&(end..end)), Some(13..13));
        let wrapper = source.generated.find('(').unwrap();
        assert_eq!(source.original_range(&(wrapper..end + 1)), Some(0..13));
        let generated = source.generated.find("injected").unwrap();
        assert_eq!(source.original_range(&(generated..generated + 8)), None);
        assert_eq!(source.original_range(&(0..end)), None);
    }
}
