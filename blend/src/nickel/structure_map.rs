use std::collections::HashMap;
use std::ops::Range;

use anyhow::{Context, Result};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Structural information about a record in the .ncl source
#[derive(Debug)]
pub struct RecordInfo {
    /// Byte offset of the closing `}`
    pub close_brace: usize,
    /// Fields within this record
    pub fields: Vec<FieldInfo>,
    /// Indentation level (spaces before first field, for formatting new insertions)
    pub field_indent: usize,
}

/// Structural information about a single field definition
#[derive(Debug)]
pub struct FieldInfo {
    /// Dotted field path relative to the containing from_config (e.g., "font.size")
    pub path: String,
    /// Byte range of the entire field definition (from field name to end of value,
    /// including trailing comma if present)
    pub full_range: Range<usize>,
    /// Byte range of just the value expression
    pub value_range: Range<usize>,
    /// Whether a trailing comma exists after the value
    pub has_trailing_comma: bool,
}

/// One operand of a `&` merge expression that is itself a literal record.
///
/// When the from_config value is `{a = 1} & {b = 2}`, the LHS becomes the
/// canonical `root` and each additional literal-record operand becomes a
/// `MergePartner`. Operands that aren't literal records (e.g. the result of a
/// `match`) contribute nothing here — their leaves are still tracked via
/// shadow walk for value rewriting, but structural ops (Insert/Delete) on
/// paths that live only in such operands aren't supported.
#[derive(Debug)]
pub struct MergePartner {
    pub root: RecordInfo,
    pub nested_records: HashMap<String, RecordInfo>,
}

/// Complete structural map for a from_config entry
#[derive(Debug)]
pub struct StructureMap {
    /// The root (LHS) record of the from_config value
    pub root: RecordInfo,
    /// All nested records under `root`, keyed by their field path prefix
    pub nested_records: HashMap<String, RecordInfo>,
    /// Additional literal-record operands of an outermost `&` merge
    pub merge_partners: Vec<MergePartner>,
    /// Comment byte ranges within the .ncl source.
    ///
    /// TODO: Insert/Delete in `surgical_rewrite_with_structure` does not yet
    /// consult these ranges, so deleting a field leaves its leading doc
    /// comment orphaned. See `test_delete_field_should_remove_leading_doc_comment`
    /// (an inverted `#[should_panic]` test) for the concrete gap this field
    /// is meant to close.
    #[allow(dead_code)]
    pub comments: Vec<Range<usize>>,
}

impl StructureMap {
    /// Get the record that contains a field path.
    ///
    /// For path "font.size", returns the RecordInfo for "font".
    /// For path "key", returns the root RecordInfo (Insert at top level always
    /// targets the canonical LHS).
    ///
    /// When the path doesn't exist in the root scope, falls through to merge
    /// partners. Top-level Inserts deliberately stay on the root.
    pub fn parent_record(&self, field_path: &str) -> Option<&RecordInfo> {
        if let Some(dot_pos) = field_path.rfind('.') {
            let parent_path = &field_path[..dot_pos];
            if let Some(rec) = self.nested_records.get(parent_path) {
                return Some(rec);
            }
            for partner in &self.merge_partners {
                if let Some(rec) = partner.nested_records.get(parent_path) {
                    return Some(rec);
                }
                // The partner's own root is a candidate when parent_path
                // matches no nested key but is the partner's top-level scope.
                // (Only reachable for paths whose parent IS the partner root,
                // i.e. fields directly inside the partner.)
            }
            None
        } else {
            Some(&self.root)
        }
    }

    /// Find a field by its full dotted path. Searches root scope first, then
    /// each merge partner's scope.
    pub fn find_field(&self, field_path: &str) -> Option<&FieldInfo> {
        if let Some(f) = find_field_in_scope(&self.root, &self.nested_records, field_path) {
            return Some(f);
        }
        for partner in &self.merge_partners {
            if let Some(f) = find_field_in_scope(&partner.root, &partner.nested_records, field_path)
            {
                return Some(f);
            }
        }
        None
    }
}

fn find_field_in_scope<'a>(
    root: &'a RecordInfo,
    nested: &'a HashMap<String, RecordInfo>,
    field_path: &str,
) -> Option<&'a FieldInfo> {
    for field in &root.fields {
        if field.path == field_path {
            return Some(field);
        }
    }
    for record in nested.values() {
        for field in &record.fields {
            if field.path == field_path {
                return Some(field);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// tree-sitter helpers
// ---------------------------------------------------------------------------

/// Create a tree-sitter parser configured for Nickel
fn create_parser() -> Result<tree_sitter::Parser> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_nickel::LANGUAGE.into();
    parser
        .set_language(&language)
        .map_err(|e| anyhow::anyhow!("Failed to set tree-sitter language: {}", e))?;
    Ok(parser)
}

/// Parse source with tree-sitter and return the tree
fn parse_source(source: &str) -> Result<tree_sitter::Tree> {
    let mut parser = create_parser()?;
    parser
        .parse(source, None)
        .context("tree-sitter failed to parse source")
}

// ---------------------------------------------------------------------------
// CST navigation: find from_config node
// ---------------------------------------------------------------------------

/// Navigate the CST to find the `from_config` value node for a specific file entry.
///
/// Path: root record -> field "blend" -> record -> field "files" -> array -> entry[index] -> field "from_config" -> value
fn find_from_config_node<'a>(
    root: tree_sitter::Node<'a>,
    file_entry_index: usize,
    source: &str,
) -> Result<tree_sitter::Node<'a>> {
    // The root might be wrapped in let-bindings and/or annotations.
    // Walk through until we find the top-level record.
    let top_record = find_top_record(root, source)?;

    // Find the "blend" field
    let blend_value = find_field_value_in_record(top_record, "blend", source)?
        .context("No 'blend' field found in CST")?;

    // blend's value should be a record; find "files" in it
    let blend_record = descend_to_record(blend_value, source)?;
    let files_value = find_field_value_in_record(blend_record, "files", source)?
        .context("No 'files' field found in blend")?;

    // files should be an array; get the entry at file_entry_index
    let files_array = descend_to_array(files_value, source)?;
    let entry_node = get_array_element(files_array, file_entry_index, source)?;

    // entry should be a record; find "from_config"
    let entry_record = descend_to_record(entry_node, source)?;
    let from_config_value = find_field_value_in_record(entry_record, "from_config", source)?
        .context("No 'from_config' field found in file entry")?;

    Ok(from_config_value)
}

/// Find the first `uni_record` node by depth-first search.
///
/// The CST wraps records in many layers: `term` -> `uni_term` -> `infix_expr`
/// -> `applicative` -> `record_operand` -> `atom` -> `uni_record`.
/// For let-expressions, the body is the last named child.
/// For annotations (`{ ... } | Order`), the record is in the first operand.
fn find_top_record<'a>(
    node: tree_sitter::Node<'a>,
    _source: &str,
) -> Result<tree_sitter::Node<'a>> {
    if node.kind() == "uni_record" {
        return Ok(node);
    }

    // Recurse into named children
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Ok(r) = find_top_record(child, _source) {
            return Ok(r);
        }
    }

    anyhow::bail!(
        "Could not find top-level record in CST (found node kind: '{}')",
        node.kind()
    )
}

/// Find a named field's value node within a record node.
///
/// In the CST, a `uni_record` contains `field_decl` children, each with
/// a `field_path` and a value expression.
fn find_field_value_in_record<'a>(
    record: tree_sitter::Node<'a>,
    field_name: &str,
    source: &str,
) -> Result<Option<tree_sitter::Node<'a>>> {
    let mut cursor = record.walk();
    for child in record.children(&mut cursor) {
        // Collect field_decl nodes, including those inside last_field
        let field_decl = match child.kind() {
            "field_decl" => child,
            "last_field" => match find_child_by_kind(child, "field_decl") {
                Some(inner) => inner,
                None => continue,
            },
            _ => continue,
        };
        // Check field name and extract value
        if let Some(name) = extract_field_name(field_decl, source)
            && name == field_name
            && let Some(value) = extract_field_value(field_decl)
        {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

/// Extract the field name from a `field_decl` node.
///
/// Structure: field_decl -> field_def -> field_path -> field_path_elem -> ident | str_chunks
fn extract_field_name(field_decl: tree_sitter::Node, source: &str) -> Option<String> {
    // Try field_def child first
    let field_def = find_child_by_kind(field_decl, "field_def")?;
    let field_path = find_child_by_kind(field_def, "field_path")?;
    let path_elem = find_child_by_kind(field_path, "field_path_elem")?;

    extract_ident_or_str(path_elem, source)
}

/// Extract all path segments from a field_decl (for dotted paths like `font.size`)
fn extract_field_path_segments(field_decl: tree_sitter::Node, source: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let field_def = match find_child_by_kind(field_decl, "field_def") {
        Some(fd) => fd,
        None => return segments,
    };
    let field_path = match find_child_by_kind(field_def, "field_path") {
        Some(fp) => fp,
        None => return segments,
    };

    let mut cursor = field_path.walk();
    for child in field_path.children(&mut cursor) {
        if child.kind() == "field_path_elem"
            && let Some(name) = extract_ident_or_str(child, source)
        {
            segments.push(name);
        }
    }
    segments
}

/// Extract an identifier or quoted string from a field_path_elem
fn extract_ident_or_str(path_elem: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = path_elem.walk();
    for child in path_elem.children(&mut cursor) {
        match child.kind() {
            "ident" => {
                let text = &source[child.start_byte()..child.end_byte()];
                return Some(text.to_string());
            }
            "str_chunks" | "str_chunks_single" => {
                // Quoted field name: extract inner text (without quotes)
                return extract_string_content(child, source);
            }
            _ => {}
        }
    }
    None
}

/// Extract the string content from a str_chunks or str_chunks_single node
fn extract_string_content(str_node: tree_sitter::Node, source: &str) -> Option<String> {
    // The string literal includes quotes; we need the inner content
    let text = &source[str_node.start_byte()..str_node.end_byte()];
    // Remove surrounding quotes
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        Some(text[1..text.len() - 1].to_string())
    } else {
        Some(text.to_string())
    }
}

/// Extract the value expression from a field_decl or field_def node
fn extract_field_value(field_decl: tree_sitter::Node) -> Option<tree_sitter::Node> {
    // Structure: field_decl -> field_def -> ... -> value
    // The value is typically the last named child of field_def that isn't field_path
    let field_def = find_child_by_kind(field_decl, "field_def")?;

    // In tree-sitter-nickel, after the `=` operator, the value is the remaining expression.
    // We look for the child after `=`.
    let mut found_eq = false;
    let mut cursor = field_def.walk();
    for child in field_def.children(&mut cursor) {
        if found_eq && child.is_named() {
            return Some(child);
        }
        if child.kind() == "=" {
            found_eq = true;
        }
    }

    // Fallback: last named child that isn't field_path
    let mut last_value = None;
    let mut cursor2 = field_def.walk();
    for child in field_def.named_children(&mut cursor2) {
        if child.kind() != "field_path" {
            last_value = Some(child);
        }
    }
    last_value
}

/// Find first child with the given kind
fn find_child_by_kind<'a>(
    node: tree_sitter::Node<'a>,
    kind: &str,
) -> Option<tree_sitter::Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|c| c.kind() == kind)
}

/// Descend through wrapper nodes (`term`, `uni_term`, `infix_expr`, etc.)
/// to find a `uni_record` node.
fn descend_to_record<'a>(
    node: tree_sitter::Node<'a>,
    _source: &str,
) -> Result<tree_sitter::Node<'a>> {
    if node.kind() == "uni_record" {
        return Ok(node);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Ok(r) = descend_to_record(child, _source) {
            return Ok(r);
        }
    }

    anyhow::bail!(
        "Expected record node, found '{}' at byte {}",
        node.kind(),
        node.start_byte()
    )
}

/// Detect whether an `infix_expr` node is a `&` merge.
fn is_merge_infix(node: tree_sitter::Node) -> bool {
    if node.kind() != "infix_expr" {
        return false;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // tree-sitter-nickel encodes `&` as: infix_b_op_6 -> &
        if child.kind() == "infix_b_op_6" {
            let mut op_cursor = child.walk();
            if child.children(&mut op_cursor).any(|c| c.kind() == "&") {
                return true;
            }
        }
    }
    false
}

/// Collect every literal-record operand reachable from `node`.
///
/// - A `uni_record` IS a literal record.
/// - A `&`-merge `infix_expr` recurses into both operands.
/// - Pass-through wrappers (`term`, `uni_term`, `applicative`, `record_operand`,
///   `atom`, plain `infix_expr`) descend into their named children.
/// - Anything else (e.g. `match_expr`, `fun_expr`, `let_expr`) is opaque to us
///   and contributes no literal records — its leaves still get rewritten via
///   shadow walk, but Insert/Delete on those paths is unsupported.
fn collect_record_operands<'a>(node: tree_sitter::Node<'a>, out: &mut Vec<tree_sitter::Node<'a>>) {
    match node.kind() {
        "uni_record" => out.push(node),
        "infix_expr" if is_merge_infix(node) => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                if child.kind() != "infix_b_op_6" {
                    collect_record_operands(child, out);
                }
            }
        }
        "term" | "uni_term" | "infix_expr" | "applicative" | "record_operand" | "atom" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                collect_record_operands(child, out);
            }
        }
        _ => {}
    }
}

/// Descend through wrapper nodes to find the `atom` node that contains
/// array brackets `[` ... `]`.
///
/// In the CST, arrays appear as:
/// `term` -> `uni_term` -> `infix_expr` -> `applicative` -> `record_operand` -> `atom`
/// where `atom` has `[`, element `term` nodes, and `]` as children.
fn descend_to_array<'a>(
    node: tree_sitter::Node<'a>,
    _source: &str,
) -> Result<tree_sitter::Node<'a>> {
    // Check if this node is the array container (has `[` child)
    if node.kind() == "atom" {
        let mut cursor = node.walk();
        let has_bracket = node.children(&mut cursor).any(|c| c.kind() == "[");
        if has_bracket {
            return Ok(node);
        }
    }

    // Recurse into named children
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Ok(r) = descend_to_array(child, _source) {
            return Ok(r);
        }
    }

    anyhow::bail!(
        "Expected array node, found '{}' at byte {}",
        node.kind(),
        node.start_byte()
    )
}

/// Get the nth element from an array `atom` node.
///
/// Array elements are `term` nodes that are named children of the `atom`.
fn get_array_element<'a>(
    array: tree_sitter::Node<'a>,
    index: usize,
    _source: &str,
) -> Result<tree_sitter::Node<'a>> {
    let mut count = 0;
    let mut cursor = array.walk();
    for child in array.named_children(&mut cursor) {
        // Array elements are `term` nodes; skip other named children
        if child.kind() == "term" {
            if count == index {
                return Ok(child);
            }
            count += 1;
        }
    }
    anyhow::bail!(
        "Array index {} out of bounds (found {} elements)",
        index,
        count
    )
}

// ---------------------------------------------------------------------------
// StructureMap building
// ---------------------------------------------------------------------------

/// Build a StructureMap for a specific from_config entry in an order.ncl file.
///
/// `source` is the .ncl file content.
/// `file_entry_index` identifies which files[] entry to map.
pub fn build_structure_map(source: &str, file_entry_index: usize) -> Result<StructureMap> {
    let tree = parse_source(source)?;
    let root_node = tree.root_node();

    let from_config_node = find_from_config_node(root_node, file_entry_index, source)?;

    // Collect every literal-record operand. For a plain record this is one
    // node; for `{...} & {...}` it's both operands; for `{...} & (match {...})`
    // it's just the LHS (the match isn't a literal record).
    let mut operands = Vec::new();
    collect_record_operands(from_config_node, &mut operands);
    if operands.is_empty() {
        anyhow::bail!("from_config value contains no literal record in CST");
    }

    let mut comments = Vec::new();
    collect_comments(root_node, &mut comments);

    let mut nested_records = HashMap::new();
    let root = build_record_info(operands[0], "", source, &mut nested_records)?;

    let mut merge_partners = Vec::new();
    for operand in &operands[1..] {
        let mut partner_nested = HashMap::new();
        let partner_root = build_record_info(*operand, "", source, &mut partner_nested)?;
        merge_partners.push(MergePartner {
            root: partner_root,
            nested_records: partner_nested,
        });
    }

    Ok(StructureMap {
        root,
        nested_records,
        merge_partners,
        comments,
    })
}

/// Build RecordInfo for a record node, recursing into nested records
fn build_record_info(
    record: tree_sitter::Node,
    path_prefix: &str,
    source: &str,
    nested_records: &mut HashMap<String, RecordInfo>,
) -> Result<RecordInfo> {
    let close_brace = find_close_brace(record)?;

    let mut fields = Vec::new();
    let mut field_indent = 0;

    // Collect field_decl nodes from the record.
    // In tree-sitter-nickel, fields with trailing commas are direct `field_decl`
    // children of `uni_record`. The last field (without trailing comma) is wrapped
    // in a `last_field` node: `last_field` -> `field_decl`.
    let mut field_decls: Vec<tree_sitter::Node> = Vec::new();
    let mut cursor = record.walk();
    for child in record.children(&mut cursor) {
        match child.kind() {
            "field_decl" => field_decls.push(child),
            "last_field" => {
                // Unwrap: last_field -> field_decl
                if let Some(inner) = find_child_by_kind(child, "field_decl") {
                    field_decls.push(inner);
                }
            }
            _ => {}
        }
    }

    let mut first_field = true;
    for field_decl in &field_decls {
        let segments = extract_field_path_segments(*field_decl, source);
        if segments.is_empty() {
            continue;
        }

        let field_name = segments.join(".");
        let full_path = if path_prefix.is_empty() {
            field_name.clone()
        } else {
            format!("{}.{}", path_prefix, field_name)
        };

        // Compute indentation from the first field
        if first_field {
            field_indent = compute_indent(source, field_decl.start_byte());
            first_field = false;
        }

        // Get the value node
        let value_node = extract_field_value(*field_decl);

        // Determine full range and value range
        let field_start = field_decl.start_byte();
        let mut field_end = field_decl.end_byte();

        let value_range = if let Some(vn) = value_node {
            vn.start_byte()..vn.end_byte()
        } else {
            field_start..field_end
        };

        // Check for trailing comma
        let (has_trailing_comma, comma_offset) = detect_trailing_comma(source, field_end);
        if has_trailing_comma {
            field_end = comma_offset.unwrap() + 1;
        }

        fields.push(FieldInfo {
            path: full_path.clone(),
            full_range: field_start..field_end,
            value_range,
            has_trailing_comma,
        });

        // If value is a record, recurse to build nested RecordInfo
        if let Some(vn) = value_node
            && let Ok(nested_rec) = descend_to_record(vn, source)
            && let Ok(nested_info) =
                build_record_info(nested_rec, &full_path, source, nested_records)
        {
            nested_records.insert(full_path, nested_info);
        }
    }

    Ok(RecordInfo {
        close_brace,
        fields,
        field_indent,
    })
}

/// Find the closing brace byte offset of a record node
fn find_close_brace(record: tree_sitter::Node) -> Result<usize> {
    let mut close = None;
    let mut cursor = record.walk();
    for child in record.children(&mut cursor) {
        if child.kind() == "}" {
            close = Some(child.start_byte());
        }
    }
    close.context("Record has no closing brace in CST")
}

/// Detect a trailing comma after a field definition
fn detect_trailing_comma(source: &str, after_byte: usize) -> (bool, Option<usize>) {
    let rest = &source[after_byte..];
    for (i, ch) in rest.char_indices() {
        match ch {
            ',' => return (true, Some(after_byte + i)),
            ' ' | '\t' | '\n' | '\r' => continue,
            _ => return (false, None),
        }
    }
    (false, None)
}

/// Compute the indentation (number of spaces) at a given byte offset
fn compute_indent(source: &str, offset: usize) -> usize {
    let before = &source[..offset];
    let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line_prefix = &source[line_start..offset];
    line_prefix.len() - line_prefix.trim_start().len()
}

/// Collect all comment ranges from the CST
fn collect_comments(node: tree_sitter::Node, comments: &mut Vec<Range<usize>>) {
    if node.kind() == "comment" {
        comments.push(node.start_byte()..node.end_byte());
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_comments(child, comments);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helper to dump CST for debugging --

    #[allow(dead_code)]
    fn dump_cst(node: tree_sitter::Node, source: &str, indent: usize) {
        let prefix = "  ".repeat(indent);
        let text = &source[node.start_byte()..node.end_byte()];
        let display = if text.len() > 60 {
            format!("{}...", &text[..57])
        } else {
            text.to_string()
        };
        eprintln!(
            "{}{} [{}-{}] {:?}",
            prefix,
            node.kind(),
            node.start_byte(),
            node.end_byte(),
            display
        );
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            dump_cst(child, source, indent + 1);
        }
    }

    // -----------------------------------------------------------------------
    // a. Basic record extraction
    // -----------------------------------------------------------------------

    #[test]
    fn test_basic_record_extraction() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          a = 1,
          b = "hello",
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        // Root record should have two fields
        assert_eq!(map.root.fields.len(), 2);

        // Check field paths
        assert_eq!(map.root.fields[0].path, "a");
        assert_eq!(map.root.fields[1].path, "b");

        // Check value ranges point to actual values
        let a_val = &source[map.root.fields[0].value_range.clone()];
        assert_eq!(a_val, "1");

        let b_val = &source[map.root.fields[1].value_range.clone()];
        assert_eq!(b_val, "\"hello\"");
    }

    // -----------------------------------------------------------------------
    // b. Nested records
    // -----------------------------------------------------------------------

    #[test]
    fn test_nested_records() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          font = {
            size = 12,
            family = "Mono",
          },
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        // Root should have one field: "font"
        assert_eq!(map.root.fields.len(), 1);
        assert_eq!(map.root.fields[0].path, "font");

        // There should be a nested record at path "font"
        assert!(
            map.nested_records.contains_key("font"),
            "Should have nested record for 'font'"
        );

        let font_record = &map.nested_records["font"];
        assert_eq!(font_record.fields.len(), 2);
        assert_eq!(font_record.fields[0].path, "font.size");
        assert_eq!(font_record.fields[1].path, "font.family");

        // Verify nested value ranges
        let size_val = &source[font_record.fields[0].value_range.clone()];
        assert_eq!(size_val, "12");
    }

    // -----------------------------------------------------------------------
    // c. Dotted field paths
    // -----------------------------------------------------------------------

    #[test]
    fn test_dotted_field_paths() {
        // Nickel dotted path syntax: `font.size = 12` is syntactic sugar
        // for `font = { size = 12 }`. In the CST, the field_path has
        // multiple field_path_elem children.
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          font.size = 12,
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        // The field should have path "font.size" (joined segments)
        assert_eq!(map.root.fields.len(), 1);
        assert_eq!(map.root.fields[0].path, "font.size");
    }

    // -----------------------------------------------------------------------
    // d. Quoted field names
    // -----------------------------------------------------------------------

    #[test]
    fn test_quoted_field_names() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          "$schema" = "https://example.com",
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        assert_eq!(map.root.fields.len(), 1);
        assert_eq!(map.root.fields[0].path, "$schema");
    }

    // -----------------------------------------------------------------------
    // e. Trailing comma detection
    // -----------------------------------------------------------------------

    #[test]
    fn test_trailing_comma_detection() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          a = 1,
          b = 2,
          c = 3
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        assert_eq!(map.root.fields.len(), 3, "Should have 3 fields");

        // First two fields should have trailing commas, last should not
        assert!(
            map.root.fields[0].has_trailing_comma,
            "field 'a' should have trailing comma"
        );

        assert!(
            map.root.fields[1].has_trailing_comma,
            "field 'b' should have trailing comma"
        );

        // 'c' has no trailing comma (next char after value should be newline or })
        assert!(
            !map.root.fields[2].has_trailing_comma,
            "field 'c' should NOT have trailing comma"
        );
    }

    // -----------------------------------------------------------------------
    // f. Comment preservation
    // -----------------------------------------------------------------------

    #[test]
    fn test_comment_preservation() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        # This is a comment
        from_config = {
          # Field comment
          a = 1,
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        // Should have collected comment ranges
        assert!(
            !map.comments.is_empty(),
            "Should have found comments in CST"
        );

        // Verify comment text
        let has_field_comment = map
            .comments
            .iter()
            .any(|r| source[r.clone()].contains("Field comment"));
        assert!(has_field_comment, "Should find 'Field comment'");
    }

    /// Pins the gap that `StructureMap::comments` exists to close.
    ///
    /// `surgical_rewrite_with_structure` deletes a field by removing the byte
    /// range from line-start to the next newline. A doc comment immediately
    /// above the deleted field survives — but it now misleadingly precedes
    /// the next field. The intended fix is for the rewrite path to consult
    /// `structure.comments`, detect comments attached to the field being
    /// deleted, and remove them too.
    ///
    /// Inverted-status test: marked `#[should_panic]` so the suite "passes"
    /// while the gap exists. The day the safety check lands, the inner
    /// `assert!` will succeed, the test will stop panicking, and
    /// `should_panic` will fail loudly — at which point flip this to a
    /// regular `#[test]` that asserts the desired behavior directly.
    #[test]
    #[should_panic(expected = "Doc comment for deleted field should be removed too")]
    fn test_delete_field_should_remove_leading_doc_comment() {
        use super::super::ast_utils::{FieldEdit, surgical_rewrite_with_structure};

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          # Doc comment describing field a
          a = 1,
          b = 2,
        },
      },
    ],
  },
}"#;

        let structure = build_structure_map(source, 0).unwrap();
        let edits = vec![FieldEdit::Delete {
            path: "a".to_string(),
        }];
        let result = surgical_rewrite_with_structure(source, &structure, &[], &edits, 5).unwrap();

        assert!(
            !result.contains("Doc comment describing field a"),
            "Doc comment for deleted field should be removed too. Got:\n{result}"
        );
    }

    // -----------------------------------------------------------------------
    // g. Real order file: starship
    // -----------------------------------------------------------------------

    #[test]
    fn test_real_starship_order() {
        let orders_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("orders");
        let starship_path = orders_dir.join("starship/order.ncl");
        if !starship_path.exists() {
            eprintln!("Skipping real order test: {:?} not found", starship_path);
            return;
        }

        let source = std::fs::read_to_string(&starship_path).unwrap();
        let map = build_structure_map(&source, 0).unwrap();

        // Starship has known fields
        let field_paths: Vec<&str> = map.root.fields.iter().map(|f| f.path.as_str()).collect();
        assert!(
            field_paths.contains(&"$schema"),
            "Starship should have $schema field, found: {:?}",
            field_paths
        );
        assert!(
            field_paths.contains(&"command_timeout"),
            "Starship should have command_timeout field"
        );
        assert!(
            field_paths.contains(&"git_branch"),
            "Starship should have git_branch field"
        );
        assert!(
            field_paths.contains(&"shell"),
            "Starship should have shell field"
        );

        // Check nested records
        assert!(
            map.nested_records.contains_key("shell"),
            "Should have nested record for 'shell'"
        );
        let shell_record = &map.nested_records["shell"];
        let shell_fields: Vec<&str> = shell_record
            .fields
            .iter()
            .map(|f| f.path.as_str())
            .collect();
        assert!(
            shell_fields.contains(&"shell.disabled"),
            "shell should have disabled field, found: {:?}",
            shell_fields
        );

        // Verify comment collection (starship has Nerd Fonts comment)
        let has_nerd_comment = map
            .comments
            .iter()
            .any(|r| source[r.clone()].contains("Nerd Fonts"));
        assert!(has_nerd_comment, "Should find Nerd Fonts comment");

        // Field indent should be reasonable (likely 10 spaces for starship)
        assert!(map.root.field_indent > 0, "Field indent should be non-zero");
    }

    // -----------------------------------------------------------------------
    // h. Indentation inference
    // -----------------------------------------------------------------------

    #[test]
    fn test_indentation_inference() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          a = 1,
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        // The field "a" is indented by 10 spaces (5 levels of 2-space indent)
        assert_eq!(
            map.root.field_indent, 10,
            "Expected 10 spaces indent for field 'a'"
        );
    }

    // -----------------------------------------------------------------------
    // i. Multiple file entries
    // -----------------------------------------------------------------------

    #[test]
    fn test_multiple_file_entries() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "first.toml",
        from_config = {
          x = 1,
        },
      },
      {
        name = "second.toml",
        from_config = {
          y = 2,
          z = 3,
        },
      },
    ],
  },
}"#;
        // Map the first entry
        let map0 = build_structure_map(source, 0).unwrap();
        assert_eq!(map0.root.fields.len(), 1);
        assert_eq!(map0.root.fields[0].path, "x");

        // Map the second entry
        let map1 = build_structure_map(source, 1).unwrap();
        assert_eq!(map1.root.fields.len(), 2);
        assert_eq!(map1.root.fields[0].path, "y");
        assert_eq!(map1.root.fields[1].path, "z");
    }

    // -----------------------------------------------------------------------
    // CST exploration test (debug helper)
    // -----------------------------------------------------------------------

    #[test]
    fn test_cst_exploration() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          a = 1,
          b = "hello",
        },
      },
    ],
  },
}"#;
        let tree = parse_source(source).unwrap();
        let root = tree.root_node();
        // Root is "term" in tree-sitter-nickel (not "source_file")
        assert_eq!(root.kind(), "term");
        assert!(!root.has_error(), "CST should parse without errors");
    }

    // -----------------------------------------------------------------------
    // Query API tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parent_record_lookup() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          section = {
            key = "val",
          },
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        // Parent of "section.key" should be the "section" nested record
        let parent = map.parent_record("section.key").unwrap();
        // It should be the nested record for "section"
        assert!(!parent.fields.is_empty());
        assert_eq!(parent.fields[0].path, "section.key");

        // Parent of "section" should be the root
        let root_parent = map.parent_record("section").unwrap();
        assert_eq!(root_parent.fields[0].path, "section");
    }

    #[test]
    fn test_find_field() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          a = 1,
          b = {
            c = 2,
          },
        },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();
        let field_a = map.find_field("a").unwrap();
        assert_eq!(field_a.path, "a");

        let field_c = map.find_field("b.c").unwrap();
        assert_eq!(field_c.path, "b.c");

        assert!(map.find_field("nonexistent").is_none());
    }

    // -----------------------------------------------------------------------
    // Merge-aware structure map: `&` between literal records
    // -----------------------------------------------------------------------

    #[test]
    fn test_merge_two_literal_records() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = { a = 1, b = 2 } & { c = 3 },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        // Root (LHS) has a, b
        let root_paths: Vec<&str> = map.root.fields.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(root_paths, vec!["a", "b"]);

        // Partner (RHS) has c
        assert_eq!(map.merge_partners.len(), 1);
        let partner_paths: Vec<&str> = map.merge_partners[0]
            .root
            .fields
            .iter()
            .map(|f| f.path.as_str())
            .collect();
        assert_eq!(partner_paths, vec!["c"]);

        // find_field sees both scopes
        assert!(map.find_field("a").is_some());
        assert!(map.find_field("c").is_some());

        // RHS field's value range points at "3"
        let c_field = map.find_field("c").unwrap();
        assert_eq!(&source[c_field.value_range.clone()], "3");
    }

    #[test]
    fn test_merge_literal_with_nested_records() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = { x = { a = 1 } } & { y = { b = 2 } },
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        assert_eq!(map.merge_partners.len(), 1);
        // x.a lives in root scope, y.b lives in partner scope
        let xa = map.find_field("x.a").unwrap();
        assert_eq!(&source[xa.value_range.clone()], "1");
        let yb = map.find_field("y.b").unwrap();
        assert_eq!(&source[yb.value_range.clone()], "2");

        // parent_record("y.b") finds the partner's "y" subrecord
        let parent = map.parent_record("y.b").unwrap();
        assert!(parent.fields.iter().any(|f| f.path == "y.b"));
    }

    #[test]
    fn test_merge_with_match_rhs_skips_match() {
        // The RHS is a `match` expression, not a literal record. We should
        // collect ONLY the LHS as `root`; the match's branches must NOT leak
        // into the structure map (their applicability is conditional).
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config =
          { a = 1 }
          & (
            "darwin"
            |> match {
              "linux" => { b = 2 },
              _ => {},
            }
          ),
      },
    ],
  },
}"#;
        let map = build_structure_map(source, 0).unwrap();

        // Only LHS is captured structurally.
        assert!(map.merge_partners.is_empty());
        let root_paths: Vec<&str> = map.root.fields.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(root_paths, vec!["a"]);

        // `b` is reachable via shadow walk's leaf spans, but NOT via the
        // structure map (Insert/Delete on `b` is intentionally unsupported).
        assert!(map.find_field("b").is_none());
    }

    #[test]
    fn test_merge_real_alacritty_order() {
        let orders_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("orders");
        let path = orders_dir.join("alacritty/order.ncl");
        if !path.exists() {
            eprintln!("Skipping real alacritty test: {:?} not found", path);
            return;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let map = build_structure_map(&source, 0).unwrap();

        // alacritty's from_config is `{ window=..., font=... } & (match ...)`.
        // We expect window/font to live on root and merge_partners to be empty.
        let root_paths: Vec<&str> = map.root.fields.iter().map(|f| f.path.as_str()).collect();
        assert!(root_paths.contains(&"window"));
        assert!(root_paths.contains(&"font"));
        assert!(map.merge_partners.is_empty());
    }
}
