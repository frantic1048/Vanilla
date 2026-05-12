use std::collections::HashMap;

use anyhow::{Context, Result};
use nickel_lang_parser::{
    ErrorTolerantParser,
    ast::{
        Ast, AstAlloc, Node, StringChunk,
        pattern::{ConstantPatternData, PatternData},
        primop::PrimOp,
        record::FieldDef,
    },
    files::Files,
    grammar,
    lexer::Lexer,
};

use crate::metadata::Metadata;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of analyzing a from_config value for rewritability
#[allow(dead_code)]
pub enum RewriteResult {
    /// All fields reach rewritable leaf values
    Rewritable { leaf_spans: Vec<LeafSpan> },
    /// Some fields rewritable, some not
    Partial {
        rewritable: Vec<LeafSpan>,
        non_rewritable: Vec<NonRewritableField>,
    },
    /// Nothing is rewritable (or no from_config found)
    NotRewritable,
}

impl RewriteResult {
    #[cfg(test)]
    pub fn is_rewritable(&self) -> bool {
        matches!(self, RewriteResult::Rewritable { .. })
    }

    pub fn has_any_rewritable(&self) -> bool {
        matches!(
            self,
            RewriteResult::Rewritable { .. } | RewriteResult::Partial { .. }
        )
    }

    /// Get all rewritable leaf spans
    pub fn rewritable_spans(&self) -> &[LeafSpan] {
        match self {
            RewriteResult::Rewritable { leaf_spans } => leaf_spans,
            RewriteResult::Partial { rewritable, .. } => rewritable,
            RewriteResult::NotRewritable => &[],
        }
    }

    /// Get all non-rewritable fields
    pub fn non_rewritable_fields(&self) -> &[NonRewritableField] {
        match self {
            RewriteResult::Partial { non_rewritable, .. } => non_rewritable,
            _ => &[],
        }
    }
}

/// A leaf value's source location, resolved through the shadow walk.
/// Points to the exact byte range of the value to replace — whether it's
/// at the top level or inside a conditional branch.
#[derive(Debug, Clone)]
pub struct LeafSpan {
    /// Field name (key path)
    pub name: String,
    /// Byte offset of the leaf value (from TermPos)
    pub value_start: usize,
    /// End byte offset
    pub value_end: usize,
    /// Trail of conditions followed to reach this value
    pub branch_context: Vec<String>,
}

/// A field whose value cannot be auto-pulled
#[allow(dead_code)]
pub struct NonRewritableField {
    /// Field name
    pub name: String,
    /// Why it can't be rewritten
    pub reason: String,
    /// Conditions followed before reaching the non-rewritable node
    pub branch_context: Vec<String>,
}

// ---------------------------------------------------------------------------
// Shadow walk: context-aware AST analysis
// ---------------------------------------------------------------------------

/// Perform a context-aware walk of the from_config value, following active
/// branches through conditionals using runtime metadata.
///
/// Returns a `RewriteResult` indicating which fields can be surgically rewritten.
fn find_rewritable_value<'ast>(
    ast: &Ast<'ast>,
    metadata: &Metadata,
    context: &mut Vec<String>,
) -> SingleFieldResult {
    match &ast.node {
        // Base cases: plain literals are always rewritable
        Node::Null | Node::Bool(_) | Node::Number(_) | Node::String(_) => {
            if let Some(span) = ast.pos.into_opt() {
                SingleFieldResult::Rewritable {
                    value_start: span.start.into(),
                    value_end: span.end.into(),
                    branch_context: context.clone(),
                }
            } else {
                SingleFieldResult::NotRewritable {
                    reason: "no source position".to_string(),
                    branch_context: context.clone(),
                }
            }
        }

        // String chunks: only if all chunks are literals (no interpolation)
        Node::StringChunks(chunks) => {
            let all_literal = chunks.iter().all(|c| matches!(c, StringChunk::Literal(_)));
            if all_literal {
                if let Some(span) = ast.pos.into_opt() {
                    SingleFieldResult::Rewritable {
                        value_start: span.start.into(),
                        value_end: span.end.into(),
                        branch_context: context.clone(),
                    }
                } else {
                    SingleFieldResult::NotRewritable {
                        reason: "no source position".to_string(),
                        branch_context: context.clone(),
                    }
                }
            } else {
                SingleFieldResult::NotRewritable {
                    reason: "string with interpolation".to_string(),
                    branch_context: context.clone(),
                }
            }
        }

        // Record: recurse per field (handled at the from_config level, not here)
        // When we reach a record as a field value, treat it as a rewritable unit
        Node::Record(_) | Node::Array(_) => {
            if let Some(span) = ast.pos.into_opt() {
                SingleFieldResult::Rewritable {
                    value_start: span.start.into(),
                    value_end: span.end.into(),
                    branch_context: context.clone(),
                }
            } else {
                SingleFieldResult::NotRewritable {
                    reason: "no source position".to_string(),
                    branch_context: context.clone(),
                }
            }
        }

        // Match expression applied to an argument: metadata.field |> match { ... }
        Node::App { head, args } if matches!(head.node, Node::Match(_)) => {
            if let Node::Match(m) = &head.node {
                // Try to resolve the argument against metadata
                if let Some(arg) = args.first() {
                    if let Some(resolved) = try_resolve_metadata_field(&arg.node, metadata) {
                        // Find the matching branch
                        for branch in m.branches {
                            if match_pattern(&branch.pattern.data, &resolved) {
                                let condition_desc = format_match_context(&arg.node, &resolved);
                                context.push(condition_desc);
                                return find_rewritable_value(&branch.body, metadata, context);
                            }
                        }
                        SingleFieldResult::NotRewritable {
                            reason: format!("no match branch for value \"{}\"", resolved),
                            branch_context: context.clone(),
                        }
                    } else {
                        SingleFieldResult::NotRewritable {
                            reason: "cannot resolve match argument against metadata".to_string(),
                            branch_context: context.clone(),
                        }
                    }
                } else {
                    SingleFieldResult::NotRewritable {
                        reason: "match applied without argument".to_string(),
                        branch_context: context.clone(),
                    }
                }
            } else {
                unreachable!()
            }
        }

        // If-then-else
        Node::IfThenElse {
            cond,
            then_branch,
            else_branch,
        } => {
            if let Some(result) = try_evaluate_condition(&cond.node, metadata) {
                let branch_name = if result { "then" } else { "else" };
                let condition_desc = format!("if condition → {}", branch_name);
                context.push(condition_desc);
                let active = if result { then_branch } else { else_branch };
                find_rewritable_value(active, metadata, context)
            } else {
                SingleFieldResult::NotRewritable {
                    reason: "cannot evaluate if condition".to_string(),
                    branch_context: context.clone(),
                }
            }
        }

        // Everything else: not rewritable
        _ => SingleFieldResult::NotRewritable {
            reason: "unsupported expression type".to_string(),
            branch_context: context.clone(),
        },
    }
}

/// Result for a single field's shadow walk
enum SingleFieldResult {
    Rewritable {
        value_start: usize,
        value_end: usize,
        branch_context: Vec<String>,
    },
    NotRewritable {
        reason: String,
        branch_context: Vec<String>,
    },
}

/// Try to resolve a metadata field access from an AST node.
///
/// Handles the pattern: `PrimOpApp { op: RecordStatAccess("field"), args: [Var("metadata")] }`
fn try_resolve_metadata_field(node: &Node, metadata: &Metadata) -> Option<String> {
    if let Node::PrimOpApp { op, args } = node
        && let PrimOp::RecordStatAccess(field_ident) = op
    {
        // Check that the record being accessed is `metadata`
        if let Some(arg) = args.first()
            && let Node::Var(var_ident) = &arg.node
            && var_ident.label() == "metadata"
        {
            let field_name = field_ident.label();
            return match field_name {
                "os" => Some(metadata.os.clone()),
                "arch" => Some(metadata.arch.clone()),
                "hostname" => Some(metadata.hostname.clone()),
                "desktop" => metadata.desktop.clone(),
                "user" => Some(metadata.user.clone()),
                "home" => Some(metadata.home.to_string_lossy().to_string()),
                _ => None,
            };
        }
    }
    None
}

/// Try to evaluate a simple boolean condition against metadata.
///
/// Handles: `PrimOpApp { op: Eq, args: [metadata_access, String("value")] }`
fn try_evaluate_condition(node: &Node, metadata: &Metadata) -> Option<bool> {
    if let Node::PrimOpApp { op, args } = node
        && let PrimOp::Eq = op
        && args.len() == 2
    {
        // Try both orderings: metadata.field == "value" and "value" == metadata.field
        if let Some(meta_val) = try_resolve_metadata_field(&args[0].node, metadata)
            && let Some(lit_val) = try_extract_string_literal(&args[1].node)
        {
            return Some(meta_val == lit_val);
        }
        if let Some(meta_val) = try_resolve_metadata_field(&args[1].node, metadata)
            && let Some(lit_val) = try_extract_string_literal(&args[0].node)
        {
            return Some(meta_val == lit_val);
        }
    }
    // Could extend to handle BoolAnd, BoolOr, etc. in the future
    None
}

/// Extract a string literal from an AST node
fn try_extract_string_literal<'a>(node: &'a Node) -> Option<&'a str> {
    match node {
        Node::String(s) => Some(s),
        Node::StringChunks(chunks) if chunks.len() == 1 => {
            if let StringChunk::Literal(s) = &chunks[0] {
                Some(s.as_str())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check if a Nickel pattern matches a string value
fn match_pattern(pattern: &PatternData, value: &str) -> bool {
    match pattern {
        PatternData::Wildcard => true,
        PatternData::Any(_) => true,
        PatternData::Constant(cp) => match &cp.data {
            ConstantPatternData::String(s) => *s == value,
            _ => false,
        },
        _ => false,
    }
}

/// Format a human-readable description of a match condition
fn format_match_context(arg_node: &Node, resolved_value: &str) -> String {
    if let Node::PrimOpApp { op, .. } = arg_node
        && let PrimOp::RecordStatAccess(field) = op
    {
        return format!("{} == \"{}\"", field.label(), resolved_value);
    }
    format!("matched \"{}\"", resolved_value)
}

// ---------------------------------------------------------------------------
// Public API: locate and analyze from_config
// ---------------------------------------------------------------------------

/// Parse a .ncl file and analyze the from_config field for a specific file entry.
///
/// Uses a context-aware shadow walk with runtime metadata to determine which
/// field values can be surgically rewritten (including values inside conditional branches).
pub fn locate_from_config(
    source: &str,
    file_entry_index: usize,
    metadata: &Metadata,
) -> Result<RewriteResult> {
    let alloc = AstAlloc::new();
    let mut files = Files::empty();
    let file_id = files.add("order.ncl", source);
    let lexer = Lexer::new(source);

    let parser = grammar::TermParser::new();
    let ast = parser
        .parse_strict(&alloc, file_id, lexer)
        .map_err(|e| anyhow::anyhow!("Failed to parse .ncl file: {:?}", e))?;

    // Navigate to from_config
    let root_record = unwrap_to_record(&ast)?;

    let blend_field = find_field(root_record.field_defs, "blend")
        .context("No 'blend' field found in order.ncl")?;
    let blend_value = blend_field
        .value
        .as_ref()
        .context("'blend' field has no value")?;
    let blend_record = match &blend_value.node {
        Node::Record(r) => *r,
        _ => anyhow::bail!("'blend' field is not a record"),
    };

    let files_field =
        find_field(blend_record.field_defs, "files").context("No 'files' field found in blend")?;
    let files_value = files_field
        .value
        .as_ref()
        .context("'files' field has no value")?;
    let files_array = match &files_value.node {
        Node::Array(arr) => *arr,
        _ => anyhow::bail!("'files' is not an array"),
    };

    let entry_ast = files_array
        .get(file_entry_index)
        .context("file_entry_index out of bounds")?;
    let entry_record = match &entry_ast.node {
        Node::Record(r) => *r,
        _ => anyhow::bail!("file entry is not a record"),
    };

    let from_config_field = match find_field(entry_record.field_defs, "from_config") {
        Some(f) => f,
        None => return Ok(RewriteResult::NotRewritable),
    };
    let from_config_value = match &from_config_field.value {
        Some(v) => v,
        None => return Ok(RewriteResult::NotRewritable),
    };

    // Collect fields from the from_config value, handling records and & merges
    let mut rewritable = Vec::new();
    let mut non_rewritable = Vec::new();

    collect_fields_from_value(
        from_config_value,
        metadata,
        &mut rewritable,
        &mut non_rewritable,
    );

    if non_rewritable.is_empty() && !rewritable.is_empty() {
        Ok(RewriteResult::Rewritable {
            leaf_spans: rewritable,
        })
    } else if !rewritable.is_empty() {
        Ok(RewriteResult::Partial {
            rewritable,
            non_rewritable,
        })
    } else {
        Ok(RewriteResult::NotRewritable)
    }
}

/// Collect rewritable/non-rewritable fields from an AST value.
/// Handles plain records, & merge expressions, and match/if expressions
/// that resolve to records.
fn collect_fields_from_value<'ast>(
    ast: &'ast Ast<'ast>,
    metadata: &Metadata,
    rewritable: &mut Vec<LeafSpan>,
    non_rewritable: &mut Vec<NonRewritableField>,
) {
    match &ast.node {
        // Plain record: walk each field
        Node::Record(record) => {
            collect_fields_from_record(record, metadata, rewritable, non_rewritable);
        }

        // & merge: walk both operands and emit a LeafSpan for every leaf.
        //
        // Nickel's `&` recursively merges records and accepts overlapping field
        // PATHS so long as the merged leaves agree (see
        // `test_nickel_merge_*` in loader.rs). Two operands may therefore both
        // contribute spans for the same dotted path; we keep both, because
        // rewriting only one would desynchronise the operands and trip the
        // strict leaf-equality check on the next evaluation.
        Node::PrimOpApp {
            op: nickel_lang_parser::ast::primop::PrimOp::Merge(_),
            args,
        } => {
            for arg in *args {
                collect_fields_from_value(arg, metadata, rewritable, non_rewritable);
            }
        }

        // Match expression applied to an argument: metadata.field |> match { ... }
        // Resolve the active branch, then collect fields from it
        Node::App { head, args } if matches!(head.node, Node::Match(_)) => {
            if let Node::Match(m) = &head.node
                && let Some(arg) = args.first()
                && let Some(resolved) = try_resolve_metadata_field(&arg.node, metadata)
            {
                for branch in m.branches {
                    if match_pattern(&branch.pattern.data, &resolved) {
                        collect_fields_from_value(
                            &branch.body,
                            metadata,
                            rewritable,
                            non_rewritable,
                        );
                        return;
                    }
                }
            }
        }

        // Annotated value (e.g., `expr | Contract`): unwrap
        Node::Annotated { inner, .. } => {
            collect_fields_from_value(inner, metadata, rewritable, non_rewritable);
        }

        // Parenthesized or other wrappers: try as a single value
        _ => {
            let mut context = Vec::new();
            if let SingleFieldResult::Rewritable {
                value_start,
                value_end,
                branch_context,
            } = find_rewritable_value(ast, metadata, &mut context)
            {
                rewritable.push(LeafSpan {
                    name: String::new(),
                    value_start,
                    value_end,
                    branch_context,
                });
            }
        }
    }
}

/// Collect fields from a record's field definitions, recursing into nested
/// records to produce dotted-path LeafSpans at the leaf level.
fn collect_fields_from_record<'ast>(
    record: &'ast nickel_lang_parser::ast::record::Record<'ast>,
    metadata: &Metadata,
    rewritable: &mut Vec<LeafSpan>,
    non_rewritable: &mut Vec<NonRewritableField>,
) {
    collect_fields_from_record_with_prefix(record, metadata, "", rewritable, non_rewritable);
}

fn collect_fields_from_record_with_prefix<'ast>(
    record: &'ast nickel_lang_parser::ast::record::Record<'ast>,
    metadata: &Metadata,
    prefix: &str,
    rewritable: &mut Vec<LeafSpan>,
    non_rewritable: &mut Vec<NonRewritableField>,
) {
    for fd in record.field_defs {
        let field_name = match fd.path_as_ident() {
            Some(id) => id.label().to_string(),
            None => continue,
        };

        let full_name = if prefix.is_empty() {
            field_name.clone()
        } else {
            format!("{}.{}", prefix, field_name)
        };

        let value = match &fd.value {
            Some(v) => v,
            None => continue,
        };

        // For nested records, recurse to produce leaf-level spans
        // First, try to resolve through match/if to see if the value is a record
        let resolved = resolve_to_value(value, metadata);
        if let Node::Record(nested) = &resolved.node {
            collect_fields_from_record_with_prefix(
                nested,
                metadata,
                &full_name,
                rewritable,
                non_rewritable,
            );
            continue;
        }

        // Not a record — treat as a leaf value
        let mut context = Vec::new();
        match find_rewritable_value(value, metadata, &mut context) {
            SingleFieldResult::Rewritable {
                value_start,
                value_end,
                branch_context,
            } => {
                rewritable.push(LeafSpan {
                    name: full_name,
                    value_start,
                    value_end,
                    branch_context,
                });
            }
            SingleFieldResult::NotRewritable {
                reason,
                branch_context,
            } => {
                non_rewritable.push(NonRewritableField {
                    name: full_name,
                    reason,
                    branch_context,
                });
            }
        }
    }
}

/// Resolve an AST value through match/if expressions using metadata,
/// returning the innermost value (which might be a record, literal, etc.)
fn resolve_to_value<'a>(ast: &'a Ast<'a>, metadata: &Metadata) -> &'a Ast<'a> {
    match &ast.node {
        // Match expression: resolve the active branch
        Node::App { head, args } if matches!(head.node, Node::Match(_)) => {
            if let Node::Match(m) = &head.node
                && let Some(arg) = args.first()
                && let Some(resolved) = try_resolve_metadata_field(&arg.node, metadata)
            {
                for branch in m.branches {
                    if match_pattern(&branch.pattern.data, &resolved) {
                        return resolve_to_value(&branch.body, metadata);
                    }
                }
            }
            ast // couldn't resolve, return as-is
        }
        // If-then-else: resolve the condition
        Node::IfThenElse {
            cond,
            then_branch,
            else_branch,
        } => {
            if let Some(result) = try_evaluate_condition(&cond.node, metadata) {
                if result {
                    resolve_to_value(then_branch, metadata)
                } else {
                    resolve_to_value(else_branch, metadata)
                }
            } else {
                ast
            }
        }
        // Annotated: unwrap
        Node::Annotated { inner, .. } => resolve_to_value(inner, metadata),
        // Everything else: return as-is
        _ => ast,
    }
}

// ---------------------------------------------------------------------------
// AST navigation helpers
// ---------------------------------------------------------------------------

/// Unwrap let-bindings, annotations, etc. to find the root record
fn unwrap_to_record<'ast>(
    ast: &'ast Ast<'ast>,
) -> Result<&'ast nickel_lang_parser::ast::record::Record<'ast>> {
    match &ast.node {
        Node::Record(r) => Ok(r),
        Node::Let { body, .. } => unwrap_to_record(body),
        Node::Annotated { inner, .. } => unwrap_to_record(inner),
        other => anyhow::bail!(
            "Expected record at top level, found {:?}",
            std::mem::discriminant(other)
        ),
    }
}

/// Find a field definition by name in a slice of field defs
fn find_field<'a, 'ast>(
    field_defs: &'a [FieldDef<'ast>],
    name: &str,
) -> Option<&'a FieldDef<'ast>> {
    field_defs.iter().find(|fd| {
        fd.path_as_ident()
            .map(|id| id.label() == name)
            .unwrap_or(false)
    })
}

// ---------------------------------------------------------------------------
// Surgical rewrite using LeafSpans
// ---------------------------------------------------------------------------

/// Resolve a dotted-path lookup in a JSON value.
///
/// Tries the literal path first (for keys that contain dots, e.g.
/// `"editor.fontSize"` in VS Code settings); falls back to descending
/// through nested objects segment by segment.
pub fn json_path_get<'a>(
    value: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(value);
    }
    if let serde_json::Value::Object(obj) = value
        && let Some(v) = obj.get(path)
    {
        return Some(v);
    }
    let mut current = value;
    for segment in path.split('.') {
        match current {
            serde_json::Value::Object(obj) => {
                current = obj.get(segment)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Perform a surgical rewrite of from_config values using shadow-walk LeafSpans.
///
/// Only rewrites fields that have changed between current and deployed JSON.
/// Uses the exact byte spans from the shadow walk (which may point inside
/// conditional branches). LeafSpan names use dotted paths for nested records,
/// so values are resolved through `json_path_get`.
pub fn surgical_rewrite(
    source: &str,
    leaf_spans: &[LeafSpan],
    current_json: &serde_json::Value,
    deployed_json: &serde_json::Value,
    base_indent: usize,
) -> Result<String> {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();

    for leaf in leaf_spans {
        let current_val = json_path_get(current_json, &leaf.name);
        let deployed_val = json_path_get(deployed_json, &leaf.name);
        if let (Some(cur), Some(dep)) = (current_val, deployed_val)
            && cur != dep
        {
            let new_value = json_to_nickel(dep, base_indent + 1);
            edits.push((leaf.value_start, leaf.value_end, new_value));
        }
        // Additions (key in deployed but not current) are harder with shadow walk
        // since we'd need to insert inside potentially conditional structures.
        // For now, additions are only supported for fully-rewritable from_config blocks.
    }

    edits.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = source.to_string();
    for (start, end, replacement) in &edits {
        result.replace_range(*start..*end, replacement);
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Extended surgical rewrite with structure-aware insert/delete
// ---------------------------------------------------------------------------

/// Describes an edit operation on a field in from_config.
#[derive(Debug)]
pub enum FieldEdit {
    /// Modify an existing field's value
    Modify {
        path: String,
        new_value: serde_json::Value,
    },
    /// Insert a new field into a record
    Insert {
        path: String,
        value: serde_json::Value,
    },
    /// Delete an existing field
    Delete { path: String },
}

/// Extended surgical rewrite that supports value modification, field insertion,
/// and field deletion.
///
/// Uses the StructureMap for structural info (record boundaries, field ranges)
/// and LeafSpans for value byte offsets (existing modify behavior).
///
/// `edits` describes what to do for each field path:
/// - Modify: change the value at an existing field (same as current behavior)
/// - Insert: add a new field to a record
/// - Delete: remove a field from a record
pub fn surgical_rewrite_with_structure(
    source: &str,
    structure: &super::structure_map::StructureMap,
    leaf_spans: &[LeafSpan],
    edits: &[FieldEdit],
    base_indent: usize,
) -> Result<String> {
    // Build path -> new value lookup for Modify edits (the user can only pull
    // one decision per dotted path, so this side is genuinely 1:1).
    let modify_lookup: HashMap<&str, &serde_json::Value> = edits
        .iter()
        .filter_map(|e| match e {
            FieldEdit::Modify { path, new_value } => Some((path.as_str(), new_value)),
            _ => None,
        })
        .collect();

    // Collect all concrete byte-range operations as (offset, delete_count, insert_text)
    // We'll sort these by offset descending to apply from back to front.
    let mut ops: Vec<(usize, usize, String)> = Vec::new();

    // Apply Modify edits by iterating leaf_spans, NOT by deduping spans by name.
    // A `&` merge can produce multiple spans for the same dotted path (one per
    // operand). Nickel requires the merged leaves agree, so rewriting only one
    // would cause the next evaluation to fail with a merge conflict.
    for leaf in leaf_spans {
        if let Some(new_value) = modify_lookup.get(leaf.name.as_str()) {
            let new_text = json_to_nickel(new_value, base_indent + 1);
            ops.push((
                leaf.value_start,
                leaf.value_end - leaf.value_start,
                new_text,
            ));
        }
    }

    for edit in edits {
        match edit {
            FieldEdit::Modify { .. } => {
                // Already handled above by iterating leaf_spans.
            }
            FieldEdit::Insert { path, value } => {
                // Determine the parent record and the field name to insert.
                // Try to find a nested parent record first; if not found, insert
                // at the root record with the full path as a quoted key.
                let (parent_record, field_name) = if let Some(dot_pos) = path.rfind('.') {
                    let parent_path = &path[..dot_pos];
                    if let Some(rec) = structure.parent_record(path) {
                        (rec, path[dot_pos + 1..].to_string())
                    } else {
                        // No nested record at parent path — insert as quoted
                        // dotted key at root (e.g., "workbench.editor.useModal")
                        let _ = parent_path; // suppress unused
                        (&structure.root, path.to_string())
                    }
                } else {
                    (&structure.root, path.to_string())
                };

                let indent_str = " ".repeat(parent_record.field_indent);
                // Determine indent level for json_to_nickel (each level = 2 spaces)
                let indent_level = parent_record.field_indent / 2;
                let nickel_value = json_to_nickel(value, indent_level);
                let formatted_key = format_nickel_key(&field_name);

                // If the record has fields and the last field has no trailing comma,
                // we need to add one before inserting
                if let Some(last_field) = parent_record.fields.last()
                    && !last_field.has_trailing_comma
                {
                    ops.push((last_field.value_range.end, 0, ",".to_string()));
                }

                // Insert the new field before the closing brace.
                // We need to check if the `}` is on the same line as the last field
                // or on its own line. If the `}` follows immediately (same line or
                // only whitespace), we insert field + newline + brace indentation.
                let brace_indent = if parent_record.field_indent >= 2 {
                    " ".repeat(parent_record.field_indent - 2)
                } else {
                    String::new()
                };

                // Check what's between the last content and the close brace
                let before_brace = &source[..parent_record.close_brace];
                let has_newline_before_brace = before_brace
                    .rfind('\n')
                    .is_some_and(|nl| source[nl..parent_record.close_brace].trim().is_empty());

                if has_newline_before_brace {
                    // `}` is already on its own line — replace the whitespace
                    // before `}` with: field + newline + brace indent
                    let nl_pos = before_brace.rfind('\n').unwrap();
                    let replace_len = parent_record.close_brace - nl_pos;
                    let text = format!(
                        "\n{}{} = {},\n{}",
                        indent_str, formatted_key, nickel_value, brace_indent
                    );
                    ops.push((nl_pos, replace_len, text));
                } else {
                    // `}` follows content directly — insert with newlines
                    let text = format!(
                        "\n{}{} = {},\n{}",
                        indent_str, formatted_key, nickel_value, brace_indent
                    );
                    ops.push((parent_record.close_brace, 0, text));
                }
            }
            FieldEdit::Delete { path } => {
                let field = structure
                    .find_field(path)
                    .with_context(|| format!("Field '{}' not found in structure map", path))?;

                // Determine what byte range to delete.
                // We want to remove the entire field definition, including the
                // preceding newline and indentation.
                let bytes = source.as_bytes();

                // Find the start of the line containing this field
                // Start deletion AFTER the preceding newline (keep it for the previous line)
                let line_start = source[..field.full_range.start]
                    .rfind('\n')
                    .map(|pos| pos + 1) // skip the \n itself
                    .unwrap_or(field.full_range.start);

                let delete_start = line_start;
                let mut delete_end = field.full_range.end;

                // Also consume trailing whitespace and newline after the field
                while delete_end < source.len()
                    && (bytes[delete_end] == b' '
                        || bytes[delete_end] == b'\t'
                        || bytes[delete_end] == b'\n')
                {
                    if bytes[delete_end] == b'\n' {
                        delete_end += 1;
                        break;
                    }
                    delete_end += 1;
                }

                ops.push((delete_start, delete_end - delete_start, String::new()));
            }
        }
    }

    // Sort ops by offset descending so we apply from back to front
    // (this ensures earlier edits don't shift the offsets of later ones)
    ops.sort_by(|a, b| b.0.cmp(&a.0));

    // Apply all operations
    let mut result = source.to_string();
    for (offset, delete_count, insert_text) in &ops {
        let end = offset + delete_count;
        result.replace_range(*offset..end, insert_text);
    }

    Ok(result)
}

/// Determine the indentation level of a from_config block by looking at the source.
pub fn detect_indent_level(source: &str, offset: usize) -> usize {
    let before = &source[..offset];
    let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line_prefix = &source[line_start..offset];
    let spaces = line_prefix.len() - line_prefix.trim_start().len();
    spaces / 2
}

// ---------------------------------------------------------------------------
// JSON ↔ Nickel serialization
// ---------------------------------------------------------------------------

/// Serialize a serde_json::Value to Nickel data literal syntax.
pub fn json_to_nickel(value: &serde_json::Value, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    let inner_indent = "  ".repeat(indent + 1);

    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", escape_nickel_string(s)),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            if arr.len() == 1 && is_simple_value(&arr[0]) {
                return format!("[{}]", json_to_nickel(&arr[0], 0));
            }
            let mut out = "[\n".to_string();
            for elem in arr.iter() {
                out.push_str(&inner_indent);
                out.push_str(&json_to_nickel(elem, indent + 1));
                out.push(',');
                out.push('\n');
            }
            out.push_str(&indent_str);
            out.push(']');
            out
        }
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                return "{}".to_string();
            }
            if map.len() <= 2 && map.values().all(is_simple_value) {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{} = {}", format_nickel_key(k), json_to_nickel(v, 0)))
                    .collect();
                return format!("{{ {} }}", pairs.join(", "));
            }
            let mut out = "{\n".to_string();
            for (k, v) in map {
                out.push_str(&inner_indent);
                out.push_str(&format_nickel_key(k));
                out.push_str(" = ");
                out.push_str(&json_to_nickel(v, indent + 1));
                out.push_str(",\n");
            }
            out.push_str(&indent_str);
            out.push('}');
            out
        }
    }
}

fn is_simple_value(value: &serde_json::Value) -> bool {
    matches!(
        value,
        serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_)
    )
}

pub fn format_nickel_key(key: &str) -> String {
    if is_valid_nickel_ident(key) {
        key.to_string()
    } else {
        format!("\"{}\"", escape_nickel_string(key))
    }
}

fn is_valid_nickel_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '\'')
}

fn escape_nickel_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if !c.is_ascii() => {
                out.push_str(&format!("\\u{{{:x}}}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_metadata(os: &str) -> Metadata {
        Metadata {
            os: os.to_string(),
            arch: "aarch64".to_string(),
            hostname: "testhost".to_string(),
            desktop: None,
            home: PathBuf::from("/home/test"),
            user: "test".to_string(),
        }
    }

    #[test]
    fn test_locate_plain_data() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          key = "value",
          number = 42,
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(
            result.is_rewritable(),
            "Plain data should be fully rewritable"
        );

        let spans = result.rewritable_spans();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].name, "key");
        assert_eq!(spans[1].name, "number");
        // Plain data should have empty branch context
        assert!(spans[0].branch_context.is_empty());
    }

    #[test]
    fn test_locate_match_expression() {
        let source = r#"let metadata = { os = "darwin", arch = "aarch64", hostname = "test", user = "test", home = "/home/test" } in {
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          font_size = metadata.os |> match {
            "darwin" => 14,
            _ => 12,
          },
          name = "hello",
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(
            result.is_rewritable(),
            "Match resolving to literal should be rewritable"
        );

        let spans = result.rewritable_spans();
        assert_eq!(spans.len(), 2);

        // font_size resolved through match
        let font_span = &spans[0];
        assert_eq!(font_span.name, "font_size");
        assert!(
            !font_span.branch_context.is_empty(),
            "Should have branch context"
        );
        assert!(font_span.branch_context[0].contains("darwin"));

        // Check the span points to "14"
        let value_text = &source[font_span.value_start..font_span.value_end];
        assert_eq!(value_text, "14");
    }

    #[test]
    fn test_locate_match_wildcard_branch() {
        let source = r#"let metadata = { os = "linux", arch = "x86_64", hostname = "test", user = "test", home = "/home/test" } in {
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          font_size = metadata.os |> match {
            "darwin" => 14,
            _ => 12,
          },
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("linux");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(result.is_rewritable());

        let spans = result.rewritable_spans();
        let font_span = &spans[0];
        let value_text = &source[font_span.value_start..font_span.value_end];
        assert_eq!(value_text, "12");
    }

    #[test]
    fn test_locate_with_variable_ref_not_rewritable() {
        let source = r#"let x = 42 in {
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          key = x,
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(!result.has_any_rewritable());
    }

    #[test]
    fn test_locate_no_from_config() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.txt",
        from_file = "test.txt",
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(matches!(result, RewriteResult::NotRewritable));
    }

    #[test]
    fn test_locate_partial_rewritability() {
        let source = r#"let base = 10 in
let metadata = { os = "darwin", arch = "aarch64", hostname = "test", user = "test", home = "/home/test" } in {
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          plain_key = "value",
          computed_key = base,
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();

        match &result {
            RewriteResult::Partial {
                rewritable,
                non_rewritable,
            } => {
                assert_eq!(rewritable.len(), 1);
                assert_eq!(rewritable[0].name, "plain_key");
                assert_eq!(non_rewritable.len(), 1);
                assert_eq!(non_rewritable[0].name, "computed_key");
            }
            other => panic!(
                "Expected Partial, got {:?}",
                matches!(other, RewriteResult::NotRewritable)
            ),
        }
    }

    #[test]
    fn test_locate_if_then_else() {
        let source = r#"let metadata = { os = "darwin", arch = "aarch64", hostname = "test", user = "test", home = "/home/test" } in {
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          font_size = if metadata.os == "darwin" then 14 else 12,
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(
            result.has_any_rewritable(),
            "if-then-else resolving to literal should be rewritable"
        );

        let spans = result.rewritable_spans();
        let value_text = &source[spans[0].value_start..spans[0].value_end];
        assert_eq!(value_text, "14");
        assert!(!spans[0].branch_context.is_empty());
    }

    #[test]
    fn test_locate_if_then_else_false_branch() {
        let source = r#"let metadata = { os = "linux", arch = "x86_64", hostname = "test", user = "test", home = "/home/test" } in {
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          font_size = if metadata.os == "darwin" then 14 else 12,
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("linux");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(result.has_any_rewritable());

        let spans = result.rewritable_spans();
        let value_text = &source[spans[0].value_start..spans[0].value_end];
        assert_eq!(value_text, "12");
    }

    #[test]
    fn test_locate_match_three_branches() {
        let source = r#"let metadata = { os = "darwin", arch = "aarch64", hostname = "test", user = "test", home = "/home/test" } in {
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          shell = metadata.os |> match {
            "darwin" => "/bin/zsh",
            "linux" => "/bin/bash",
            _ => "/bin/sh",
          },
        },
      },
    ],
  },
}"#;
        // Test darwin → "/bin/zsh"
        let meta_darwin = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta_darwin).unwrap();
        let spans = result.rewritable_spans();
        let value_text = &source[spans[0].value_start..spans[0].value_end];
        assert_eq!(value_text, "\"/bin/zsh\"");

        // Test linux → "/bin/bash"
        let meta_linux = test_metadata("linux");
        let result = locate_from_config(source, 0, &meta_linux).unwrap();
        let spans = result.rewritable_spans();
        let value_text = &source[spans[0].value_start..spans[0].value_end];
        assert_eq!(value_text, "\"/bin/bash\"");

        // Test windows → wildcard → "/bin/sh"
        let meta_win = test_metadata("windows");
        let result = locate_from_config(source, 0, &meta_win).unwrap();
        let spans = result.rewritable_spans();
        let value_text = &source[spans[0].value_start..spans[0].value_end];
        assert_eq!(value_text, "\"/bin/sh\"");
    }

    #[test]
    fn test_locate_metadata_arch() {
        let source = r#"let metadata = { os = "darwin", arch = "aarch64", hostname = "test", user = "test", home = "/home/test" } in {
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          prefix = metadata.arch |> match {
            "aarch64" => "/opt/homebrew",
            _ => "/usr/local",
          },
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();
        let spans = result.rewritable_spans();
        let value_text = &source[spans[0].value_start..spans[0].value_end];
        assert_eq!(value_text, "\"/opt/homebrew\"");
    }

    #[test]
    fn test_locate_nested_record_value() {
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          section = {
            inner_key = "inner_val",
            inner_num = 99,
          },
        },
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(result.is_rewritable());
        let spans = result.rewritable_spans();
        // Recursive walk produces leaf-level spans with dotted paths
        assert_eq!(spans.len(), 2);
        let names: Vec<&str> = spans.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"section.inner_key"));
        assert!(names.contains(&"section.inner_num"));
        let key_span = spans
            .iter()
            .find(|s| s.name == "section.inner_key")
            .unwrap();
        let value_text = &source[key_span.value_start..key_span.value_end];
        assert_eq!(value_text, "\"inner_val\"");
        let num_span = spans
            .iter()
            .find(|s| s.name == "section.inner_num")
            .unwrap();
        let num_text = &source[num_span.value_start..num_span.value_end];
        assert_eq!(num_text, "99");
    }

    #[test]
    fn test_surgical_rewrite_with_leaf_spans() {
        use serde_json::json;

        let source = "header from_config = { key = \"old_value\", num = 10 } trailer";

        let leaf_spans = vec![
            LeafSpan {
                name: "key".to_string(),
                value_start: 29, // start of "old_value"
                value_end: 40,   // end of "old_value"
                branch_context: vec![],
            },
            LeafSpan {
                name: "num".to_string(),
                value_start: 48, // start of 10
                value_end: 50,   // end of 10
                branch_context: vec![],
            },
        ];

        let current: serde_json::Value = json!({"key": "old_value", "num": 10});
        let deployed: serde_json::Value = json!({"key": "new_value", "num": 20});

        let result = surgical_rewrite(source, &leaf_spans, &current, &deployed, 0).unwrap();
        assert!(result.contains("\"new_value\""));
        assert!(result.contains("20"));
        assert!(result.contains("header"));
        assert!(result.contains("trailer"));
    }

    #[test]
    fn test_surgical_rewrite_nested_key_dotted_path() {
        use serde_json::json;
        // LeafSpans from the shadow walk use dotted paths for nested records.
        // surgical_rewrite must resolve these against nested current/deployed JSON.
        let source = "header from_config = { window = { opacity = 0.7 } } trailer";
        let value_start = source.find("0.7").unwrap();
        let value_end = value_start + 3;
        let leaf_spans = vec![LeafSpan {
            name: "window.opacity".to_string(),
            value_start,
            value_end,
            branch_context: vec![],
        }];
        let current = json!({"window": {"opacity": 0.7}});
        let deployed = json!({"window": {"opacity": 0.8}});
        let result = surgical_rewrite(source, &leaf_spans, &current, &deployed, 0).unwrap();
        assert!(
            result.contains("opacity = 0.8"),
            "Nested leaf span should be rewritten. Got:\n{}",
            result
        );
    }

    #[test]
    fn test_json_to_nickel_simple() {
        use serde_json::json;
        assert_eq!(json_to_nickel(&json!(null), 0), "null");
        assert_eq!(json_to_nickel(&json!(true), 0), "true");
        assert_eq!(json_to_nickel(&json!(42), 0), "42");
        assert_eq!(json_to_nickel(&json!("hello"), 0), "\"hello\"");
    }

    #[test]
    fn test_json_to_nickel_record() {
        use serde_json::json;
        let val = json!({"key": "value"});
        assert_eq!(json_to_nickel(&val, 0), "{ key = \"value\" }");
    }

    #[test]
    fn test_json_to_nickel_unicode_escape() {
        use serde_json::json;
        let val = json!("\u{e76f} ");
        assert_eq!(json_to_nickel(&val, 0), "\"\\u{e76f} \"");
    }

    #[test]
    fn test_escape_nickel_string() {
        assert_eq!(escape_nickel_string("hello"), "hello");
        assert_eq!(escape_nickel_string("he\"llo"), "he\\\"llo");
        assert_eq!(escape_nickel_string("\u{e76f}"), "\\u{e76f}");
    }

    #[test]
    fn test_format_nickel_key() {
        assert_eq!(format_nickel_key("simple"), "simple");
        assert_eq!(format_nickel_key("$schema"), "\"$schema\"");
    }

    #[test]
    fn test_detect_indent_level() {
        assert_eq!(detect_indent_level("    from_config = {\n", 18), 2);
        assert_eq!(detect_indent_level("top = {\n", 6), 0);
    }

    // -----------------------------------------------------------------------
    // Tests for surgical_rewrite_with_structure
    // -----------------------------------------------------------------------

    #[test]
    fn test_structure_modify_existing_value() {
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          key = "old_value",
          number = 10,
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![
            FieldEdit::Modify {
                path: "key".to_string(),
                new_value: json!("new_value"),
            },
            FieldEdit::Modify {
                path: "number".to_string(),
                new_value: json!(20),
            },
        ];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        assert!(result.contains("\"new_value\""));
        assert!(result.contains("20"));
        assert!(!result.contains("\"old_value\""));
        assert!(!result.contains(" 10,"));
    }

    #[test]
    fn test_structure_insert_flat_record() {
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

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
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Insert {
            path: "b".to_string(),
            value: json!(2),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        assert!(result.contains("a = 1"));
        assert!(result.contains("b = 2,"));
    }

    #[test]
    fn test_structure_insert_nested_record() {
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          font = {
            size = 12,
          },
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Insert {
            path: "font.family".to_string(),
            value: json!("Mono"),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        assert!(result.contains("size = 12"));
        assert!(result.contains("family = \"Mono\""));
    }

    #[test]
    fn test_structure_delete_flat_record() {
        use crate::nickel::structure_map::build_structure_map;

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          a = 1,
          b = 2,
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Delete {
            path: "b".to_string(),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        assert!(result.contains("a = 1"));
        assert!(!result.contains("b = 2"));
    }

    #[test]
    fn test_structure_delete_nested_field() {
        use crate::nickel::structure_map::build_structure_map;

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          section = {
            keep = "yes",
            remove = "no",
          },
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Delete {
            path: "section.remove".to_string(),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        assert!(result.contains("keep = \"yes\""));
        assert!(!result.contains("remove = \"no\""));
    }

    #[test]
    fn test_structure_mixed_operations() {
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          a = 1,
          b = 2,
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![
            FieldEdit::Modify {
                path: "a".to_string(),
                new_value: json!(100),
            },
            FieldEdit::Delete {
                path: "b".to_string(),
            },
            FieldEdit::Insert {
                path: "c".to_string(),
                value: json!("new"),
            },
        ];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        assert!(result.contains("a = 100"));
        assert!(!result.contains("b = 2"));
        assert!(result.contains("c = \"new\""));
    }

    #[test]
    fn test_structure_formatting_preservation() {
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          existing = "value",
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Insert {
            path: "new_field".to_string(),
            value: json!("hello"),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();

        // The inserted field should use the same indentation as existing fields
        let lines: Vec<&str> = result.lines().collect();
        let existing_line = lines.iter().find(|l| l.contains("existing")).unwrap();
        let new_line = lines.iter().find(|l| l.contains("new_field")).unwrap();

        let existing_indent = existing_line.len() - existing_line.trim_start().len();
        let new_indent = new_line.len() - new_line.trim_start().len();
        assert_eq!(
            existing_indent, new_indent,
            "Inserted field should match existing indentation"
        );
    }

    #[test]
    fn test_structure_trailing_comma_handling() {
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

        // Source without trailing comma on last field
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          a = 1
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Insert {
            path: "b".to_string(),
            value: json!(2),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        // Should have added a comma to "a = 1" and then inserted "b = 2,"
        assert!(result.contains("a = 1,") || result.contains("a = 1\n"));
        assert!(result.contains("b = 2,"));
    }

    #[test]
    fn test_structure_real_order_style() {
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

        // Simplified version of a real starship order
        let source = r#"let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        name = "starship.toml",
        from_config = {
          command_timeout = 10000,
          git_branch = {
            style = "bold bright-green",
          },
          bun = { symbol = "\u{e76f} " },
          rust = { symbol = "\u{e7a8} " },
        },
      },
    ],
  },
} | Order"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        // Insert a new symbol entry
        let edits = vec![FieldEdit::Insert {
            path: "golang".to_string(),
            value: json!({"symbol": "\u{e724} "}),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        // Original entries should still be present
        assert!(result.contains("command_timeout = 10000"));
        assert!(result.contains("bun ="));
        assert!(result.contains("rust ="));
        // New entry should be present
        assert!(result.contains("golang ="));
        // The output should still have the Order pipe at the end
        assert!(result.contains("| Order"));
    }

    #[test]
    fn test_structure_insert_dotted_key_at_root() {
        // When inserting a dotted path like "workbench.editor.useModal" and no
        // nested "workbench.editor" record exists, it should insert as a quoted
        // key at the root from_config record.
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

        let source = r#"{
  blend = {
    files = [
      {
        name = "settings.json",
        from_config = {
          "editor.fontSize" = 13,
          "editor.fontFamily" = "Mono",
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Insert {
            path: "workbench.editor.useModal".to_string(),
            value: json!("off"),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        // Should be inserted as a quoted dotted key at root level
        assert!(
            result.contains("\"workbench.editor.useModal\" = \"off\""),
            "Should insert as quoted dotted key, got:\n{}",
            result
        );
        // Existing fields preserved
        assert!(result.contains("\"editor.fontSize\" = 13"));
        assert!(result.contains("\"editor.fontFamily\" = \"Mono\""));
    }

    #[test]
    fn test_structure_delete_no_blank_line() {
        // Deleting a field should not leave a blank line behind
        use crate::nickel::structure_map::build_structure_map;

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          first = 1,
          remove_me = 2,
          last = 3,
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Delete {
            path: "remove_me".to_string(),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        assert!(result.contains("first = 1"));
        assert!(result.contains("last = 3"));
        assert!(!result.contains("remove_me"));
        // No blank line between first and last
        assert!(
            !result.contains("first = 1,\n\n"),
            "Should not leave blank line after deletion, got:\n{}",
            result
        );
    }

    #[test]
    fn test_structure_insert_preserves_closing_brace_line() {
        // Inserting a field should keep `}` on its own line
        use crate::nickel::structure_map::build_structure_map;
        use serde_json::json;

        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config = {
          existing = "value",
        },
      },
    ],
  },
}"#;
        let structure = build_structure_map(source, 0).unwrap();
        let meta = test_metadata("darwin");
        let rewrite = locate_from_config(source, 0, &meta).unwrap();
        let leaf_spans = rewrite.rewritable_spans();

        let edits = vec![FieldEdit::Insert {
            path: "new_key".to_string(),
            value: json!("hello"),
        }];

        let result =
            surgical_rewrite_with_structure(source, &structure, leaf_spans, &edits, 3).unwrap();
        assert!(result.contains("new_key = \"hello\""));
        // The closing brace should NOT be on the same line as the inserted field
        assert!(
            !result.contains("\"hello\",}"),
            "Closing brace should be on its own line, got:\n{}",
            result
        );
        // Check that `},` appears on a line by itself (with indentation)
        let lines: Vec<&str> = result.lines().collect();
        let brace_line = lines.iter().find(|l| l.trim().starts_with("},"));
        assert!(
            brace_line.is_some(),
            "Should have `}},` on its own line, got:\n{}",
            result
        );
    }

    #[test]
    fn test_locate_merge_record() {
        // Test & merge: from_config = { a = 1 } & { b = 2 }
        // Shadow walk should find fields from both sides
        let source = r#"{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config =
          {
            window = {
              opacity = 0.7,
            },
          }
          & {
            font = {
              size = 12,
            },
          },
      },
    ],
  },
}"#;
        let meta = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta).unwrap();
        assert!(
            result.has_any_rewritable(),
            "Merge record should be rewritable"
        );
        let spans = result.rewritable_spans();
        let names: Vec<&str> = spans.iter().map(|s| s.name.as_str()).collect();
        // Recursive walk produces leaf-level dotted paths
        assert!(
            names.contains(&"window.opacity"),
            "Should find 'window.opacity' from lhs, got: {:?}",
            names
        );
        assert!(
            names.contains(&"font.size"),
            "Should find 'font.size' from rhs, got: {:?}",
            names
        );
    }

    #[test]
    fn test_locate_merge_with_conditional() {
        // Test & merge where rhs is a match expression (like alacritty)
        // { window = {...} } & (metadata.os |> match { "linux" => { terminal = {...} }, _ => {} })
        let source = r#"let metadata = import "../metadata.ncl" in
{
  blend = {
    files = [
      {
        name = "test.toml",
        from_config =
          {
            window = {
              opacity = 0.7,
            },
          }
          & (
            metadata.os
            |> match {
              "linux" => { terminal = { shell = "/bin/bash" } },
              _ => {},
            }
          ),
      },
    ],
  },
}"#;
        // On darwin: rhs resolves to {} (empty record), only lhs fields
        let meta_darwin = test_metadata("darwin");
        let result = locate_from_config(source, 0, &meta_darwin).unwrap();
        assert!(result.has_any_rewritable());
        let spans = result.rewritable_spans();
        let names: Vec<&str> = spans.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"window.opacity"),
            "Should find 'window.opacity' on darwin, got: {:?}",
            names
        );
        // terminal.shell should not appear (empty record on darwin)
        assert!(
            !names.iter().any(|n| n.starts_with("terminal")),
            "Should not find 'terminal.*' on darwin, got: {:?}",
            names
        );

        // On linux: rhs resolves to { terminal = { shell = ... } }
        let meta_linux = test_metadata("linux");
        let result = locate_from_config(source, 0, &meta_linux).unwrap();
        assert!(result.has_any_rewritable());
        let spans = result.rewritable_spans();
        let names: Vec<&str> = spans.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"window.opacity"),
            "Should find 'window.opacity' on linux, got: {:?}",
            names
        );
        assert!(
            names.contains(&"terminal.shell"),
            "Should find 'terminal.shell' on linux, got: {:?}",
            names
        );
    }
}
