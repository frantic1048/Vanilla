use std::path::{Component, Path, PathBuf};

use anyhow::{Context as AnyhowContext, bail};
use similar::TextDiff;

use crate::cli::SymlinkMode;
use crate::commands::create::{starter_source, validate_order_name};
use crate::compose::{build_order, discover_orders};
use crate::context::Context;
use crate::nickel::{
    NickelEvaluator, Order, format_source, generated, normalize_order_source_path,
};
use crate::output::log;

/// Add command: copy a Target file/directory into Source and append an entry.
pub fn cmd_add(
    ctx: &Context,
    order: &str,
    path: &Path,
    prefix: Option<&str>,
    symlink: Option<SymlinkMode>,
    allow_overlap: bool,
) -> anyhow::Result<()> {
    generated::assert_orders_ready(&ctx.orders_dir)?;
    validate_order_name(order)?;

    let order_dir = ctx.orders_dir.join(order);
    let order_path = order_dir.join("order.ncl");
    let create_order = !order_dir.exists();
    if !create_order && !order_path.exists() {
        bail!(
            "Order directory exists but order.ncl is missing: {}",
            order_dir.display()
        );
    }

    let target = expand_target_path(ctx, path)?;
    let symlink_metadata = std::fs::symlink_metadata(&target)
        .with_context(|| format!("Target path not found: {}", target.display()))?;
    let is_target_symlink = symlink_metadata.file_type().is_symlink();
    if is_target_symlink && symlink.is_none() {
        bail!(
            "{} is a symlink; pass --symlink follow or --symlink preserve",
            target.display()
        );
    }

    let metadata = std::fs::metadata(&target)
        .with_context(|| format!("Failed to read Target metadata for {}", target.display()))?;
    let target_is_dir = metadata.is_dir();
    let deploy_as_symlink = symlink == Some(SymlinkMode::Preserve);

    let evaluator = NickelEvaluator::new(&ctx.metadata);
    let (raw_source, order_data) = if create_order {
        (starter_source()?, None)
    } else {
        let raw_source = std::fs::read_to_string(&order_path)
            .with_context(|| format!("Failed to read {}", order_path.display()))?;
        let order_data = evaluator.evaluate(&order_path)?;
        (raw_source, Some(order_data))
    };
    let structure = OrderSourceStructure::parse(&raw_source)?;
    let global_prefix = order_data
        .as_ref()
        .map(Order::global_prefix)
        .unwrap_or_default();
    let files_empty = order_data
        .as_ref()
        .is_none_or(|order| order.blend.files.is_empty());
    let prefix_plan = choose_prefix(ctx, global_prefix, files_empty, &structure, &target, prefix)?;
    let from_file = path_to_order_string(&prefix_plan.source_rel)?;
    normalize_order_source_path(&from_file)?;

    if let Some(order_data) = &order_data {
        ensure_no_duplicate_entry(order_data, &from_file)?;
    }

    let source_path = order_dir.join(&prefix_plan.source_rel);
    if source_path.exists() {
        bail!(
            "Source path already exists in order: {}",
            source_path.display()
        );
    }

    detect_overlaps(ctx, &target, target_is_dir, allow_overlap)?;

    let edited = edit_order_source(
        &raw_source,
        &structure,
        &from_file,
        deploy_as_symlink,
        &prefix_plan,
    )?;
    let formatted = format_source(&edited)
        .with_context(|| format!("Failed to format {}", order_path.display()))?;

    if ctx.dry_run {
        if create_order {
            log::info(&format!("Dry run: would create order '{order}'"));
        }
        log::info(&format!(
            "Dry run: would copy {} to {}",
            target.display(),
            source_path.display()
        ));
        print_source_diff(&raw_source, &formatted);
        return Ok(());
    }

    let result: anyhow::Result<()> = (|| {
        copy_target_to_source(&target, &source_path, target_is_dir)?;
        std::fs::write(&order_path, &formatted)
            .with_context(|| format!("Failed to write {}", order_path.display()))?;
        evaluator.evaluate(&order_path)?;
        Ok(())
    })();
    if result.is_err() {
        if create_order {
            let _ = std::fs::remove_dir_all(&order_dir);
        } else {
            let _ = remove_copied_source(&source_path, target_is_dir);
            let _ = std::fs::write(&order_path, &raw_source);
        }
    }
    result?;

    if create_order {
        log::success(&format!("Created order '{order}'"));
    }
    log::success(&format!("Added {} to order '{order}'", target.display()));
    Ok(())
}

struct PrefixPlan {
    prefix_literal: String,
    source_rel: PathBuf,
    write_entry_prefix: bool,
    promote_global_prefix: bool,
}

struct OrderSourceStructure {
    blend_open: usize,
    blend_close: usize,
    files_close: Option<usize>,
    has_prefix_field: bool,
}

impl OrderSourceStructure {
    fn parse(source: &str) -> anyhow::Result<Self> {
        let blend_value = find_field_value(source, "blend", 0, source.len())
            .context("Could not find `blend` record in order.ncl")?;
        let blend_open = skip_ws(source, blend_value);
        if source.as_bytes().get(blend_open) != Some(&b'{') {
            bail!("`blend` must be a record literal");
        }
        let blend_close = find_matching(source, blend_open, b'{', b'}')?;
        let has_prefix_field =
            find_field_value(source, "prefix", blend_open + 1, blend_close).is_some();
        let files_close = if let Some(files_value) =
            find_field_value(source, "files", blend_open + 1, blend_close)
        {
            let files_open = skip_ws(source, files_value);
            if source.as_bytes().get(files_open) != Some(&b'[') {
                bail!("`blend.files` must be an array literal");
            }
            Some(find_matching(source, files_open, b'[', b']')?)
        } else {
            None
        };

        Ok(Self {
            blend_open,
            blend_close,
            files_close,
            has_prefix_field,
        })
    }
}

fn choose_prefix(
    ctx: &Context,
    global_prefix: &[String],
    files_empty: bool,
    structure: &OrderSourceStructure,
    target: &Path,
    prefix: Option<&str>,
) -> anyhow::Result<PrefixPlan> {
    let is_fresh_order = files_empty && global_prefix.is_empty() && !structure.has_prefix_field;

    if let Some(prefix) = prefix {
        let expanded = ctx.expand_path_str(prefix);
        let source_rel = strip_target_prefix(target, &expanded, prefix)?;
        return Ok(PrefixPlan {
            prefix_literal: prefix.to_string(),
            source_rel,
            write_entry_prefix: !is_fresh_order,
            promote_global_prefix: is_fresh_order,
        });
    }

    for existing_prefix in global_prefix {
        let expanded = ctx.expand_path_str(existing_prefix);
        if let Ok(source_rel) = strip_target_prefix(target, &expanded, existing_prefix) {
            return Ok(PrefixPlan {
                prefix_literal: existing_prefix.clone(),
                source_rel,
                write_entry_prefix: false,
                promote_global_prefix: false,
            });
        }
    }

    if !global_prefix.is_empty() {
        bail!(
            "{} is not under the order prefix. Pass --prefix to choose a Target prefix explicitly.",
            target.display()
        );
    }

    let prefix_literal = "~".to_string();
    let expanded = ctx.expand_path_str(&prefix_literal);
    let source_rel = strip_target_prefix(target, &expanded, &prefix_literal)?;
    let promote_global_prefix = files_empty && !structure.has_prefix_field;

    Ok(PrefixPlan {
        prefix_literal,
        source_rel,
        write_entry_prefix: !promote_global_prefix,
        promote_global_prefix,
    })
}

fn expand_target_path(ctx: &Context, path: &Path) -> anyhow::Result<PathBuf> {
    let raw = path.to_string_lossy();
    if raw == "~" || raw.starts_with("~/") {
        return Ok(ctx.expand_path(path));
    }
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    bail!("Target path must be absolute or start with ~");
}

fn strip_target_prefix(
    target: &Path,
    prefix: &Path,
    prefix_literal: &str,
) -> anyhow::Result<PathBuf> {
    let rel = target.strip_prefix(prefix).with_context(|| {
        format!(
            "{} is not under prefix {}",
            target.display(),
            prefix_literal
        )
    })?;
    validate_relative_path(rel)?;
    Ok(rel.to_path_buf())
}

fn validate_relative_path(path: &Path) -> anyhow::Result<()> {
    let mut saw_component = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => saw_component = true,
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                bail!("Target path resolves outside the selected prefix");
            }
        }
    }
    if !saw_component {
        bail!("Target path must name a file or directory below the selected prefix");
    }
    Ok(())
}

fn path_to_order_string(path: &Path) -> anyhow::Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(
                part.to_str()
                    .with_context(|| format!("Path is not valid UTF-8: {}", path.display()))?,
            ),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                bail!("Source path escapes the order directory");
            }
        }
    }
    Ok(parts.join("/"))
}

fn ensure_no_duplicate_entry(order: &Order, from_file: &str) -> anyhow::Result<()> {
    let mut conflicts = Vec::new();
    for entry in &order.blend.files {
        if entry.from_file.as_deref() == Some(from_file) || entry.name == from_file {
            conflicts.push(entry.name.clone());
        }
    }

    if conflicts.is_empty() {
        Ok(())
    } else {
        conflicts.sort();
        bail!("Entry already exists in order: {}", conflicts.join(", "));
    }
}

fn detect_overlaps(
    ctx: &Context,
    new_target: &Path,
    new_is_dir: bool,
    allow_overlap: bool,
) -> anyhow::Result<()> {
    let mut conflicts = Vec::new();
    let mut orders: Vec<_> = discover_orders(&ctx.orders_dir).into_iter().collect();
    orders.sort();

    for order in orders {
        match build_order(ctx, &order) {
            Ok(results) => {
                for result in results {
                    let existing_is_dir = result
                        .source_path
                        .as_ref()
                        .is_some_and(|source| source.is_dir());
                    if paths_overlap(new_target, new_is_dir, &result.target, existing_is_dir) {
                        conflicts.push(format!(
                            "{}:{} -> {}",
                            order,
                            result.name,
                            result.target.display()
                        ));
                    }
                }
            }
            Err(err) if allow_overlap => {
                log::warn(&format!("Skipping overlap check for {order}: {err:#}"));
            }
            Err(err) => {
                bail!("Failed to check overlaps for {order}: {err:#}");
            }
        }
    }

    if conflicts.is_empty() {
        return Ok(());
    }

    conflicts.sort();
    if allow_overlap {
        log::warn(&format!(
            "Target path overlaps existing entries: {}",
            conflicts.join("; ")
        ));
        Ok(())
    } else {
        bail!(
            "Target path overlaps existing entries: {}. Pass --allow-overlap to continue.",
            conflicts.join("; ")
        );
    }
}

fn paths_overlap(
    new_path: &Path,
    new_is_dir: bool,
    existing_path: &Path,
    existing_is_dir: bool,
) -> bool {
    new_path == existing_path
        || (new_is_dir && existing_path.starts_with(new_path))
        || (existing_is_dir && new_path.starts_with(existing_path))
}

fn copy_target_to_source(target: &Path, source: &Path, is_dir: bool) -> anyhow::Result<()> {
    if let Some(parent) = source.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    if is_dir {
        copy_dir_recursive(target, source)
    } else {
        std::fs::copy(target, source).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                target.display(),
                source.display()
            )
        })?;
        Ok(())
    }
}

fn remove_copied_source(source: &Path, is_dir: bool) -> std::io::Result<()> {
    if is_dir {
        std::fs::remove_dir_all(source)
    } else {
        std::fs::remove_file(source)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst).with_context(|| format!("Failed to create {}", dst.display()))?;
    for entry in
        std::fs::read_dir(src).with_context(|| format!("Failed to read {}", src.display()))?
    {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        // Directory imports follow interior symlinks. The explicit symlink
        // policy only applies to the top-level Target path passed to add.
        let metadata = std::fs::metadata(&src_path)
            .with_context(|| format!("Failed to read metadata for {}", src_path.display()))?;
        if metadata.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn edit_order_source(
    source: &str,
    structure: &OrderSourceStructure,
    from_file: &str,
    deploy_as_symlink: bool,
    prefix_plan: &PrefixPlan,
) -> anyhow::Result<String> {
    let mut result = source.to_string();
    if let Some(files_close) = structure.files_close {
        let mut insertion = String::new();
        if !array_has_trailing_comma(source, files_close) {
            insertion.push(',');
        }
        insertion.push('\n');
        insertion.push_str(&entry_ncl(from_file, deploy_as_symlink, prefix_plan, 6));
        result.insert_str(files_close, &insertion);

        if prefix_plan.promote_global_prefix {
            insert_global_prefix_at_start(&mut result, structure, prefix_plan);
        }
        return Ok(result);
    }

    let fields = format!(
        "\n    files = [\n{}\n    ],",
        entry_ncl(from_file, deploy_as_symlink, prefix_plan, 6)
    );

    let mut insertion = String::new();
    if !record_has_trailing_comma(source, structure.blend_close) {
        insertion.push(',');
    }
    insertion.push_str(&fields);
    result.insert_str(structure.blend_close, &insertion);

    if prefix_plan.promote_global_prefix {
        insert_global_prefix_at_start(&mut result, structure, prefix_plan);
    }
    Ok(result)
}

fn insert_global_prefix_at_start(
    source: &mut String,
    structure: &OrderSourceStructure,
    prefix_plan: &PrefixPlan,
) {
    let prefix_text = format!(
        "\n    prefix = [{}],",
        nickel_string(&prefix_plan.prefix_literal)
    );
    source.insert_str(structure.blend_open + 1, &prefix_text);
}

fn entry_ncl(
    from_file: &str,
    deploy_as_symlink: bool,
    prefix_plan: &PrefixPlan,
    indent: usize,
) -> String {
    let pad = " ".repeat(indent);
    let field_pad = " ".repeat(indent + 2);
    let mut entry = format!(
        "{pad}{{\n{field_pad}from_file = {},",
        nickel_string(from_file)
    );
    if deploy_as_symlink {
        entry.push_str(&format!("\n{field_pad}symlink = true,"));
    }
    if prefix_plan.write_entry_prefix {
        entry.push_str(&format!(
            "\n{field_pad}prefix = [{}],",
            nickel_string(&prefix_plan.prefix_literal)
        ));
    }
    entry.push_str(&format!("\n{pad}}},"));
    entry
}

fn nickel_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string cannot fail")
}

fn print_source_diff(old: &str, new: &str) {
    let diff = TextDiff::from_lines(old, new);
    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        print!("{}", hunk.header());
        for change in hunk.iter_changes() {
            print!("{}{}", change.tag(), change);
        }
    }
}

fn array_has_trailing_comma(source: &str, close: usize) -> bool {
    previous_non_ws_code_byte(source, close).is_none_or(|byte| byte == b'[' || byte == b',')
}

fn record_has_trailing_comma(source: &str, close: usize) -> bool {
    previous_non_ws_code_byte(source, close).is_none_or(|byte| byte == b'{' || byte == b',')
}

fn previous_non_ws_code_byte(source: &str, offset: usize) -> Option<u8> {
    let mut previous = None;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_comment = false;

    for byte in source.as_bytes()[..offset].iter().copied() {
        if in_comment {
            if byte == b'\n' {
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
                previous = Some(byte);
            }
            continue;
        }

        match byte {
            b'#' => in_comment = true,
            b'"' => in_string = true,
            byte if byte.is_ascii_whitespace() => {}
            byte => previous = Some(byte),
        }
    }

    previous
}

fn skip_ws(source: &str, mut offset: usize) -> usize {
    while source
        .as_bytes()
        .get(offset)
        .is_some_and(|b| b.is_ascii_whitespace())
    {
        offset += 1;
    }
    offset
}

fn find_field_value(source: &str, field: &str, start: usize, end: usize) -> Option<usize> {
    let mut scanner = Scanner::new(source, start, end);
    while let Some(pos) = scanner.next_code_byte() {
        if !source[pos..].starts_with(field) || !is_field_boundary(source, pos, field.len()) {
            continue;
        }
        let after = skip_ws(source, pos + field.len());
        if source.as_bytes().get(after) != Some(&b'=') {
            continue;
        }
        return Some(after + 1);
    }
    None
}

fn is_field_boundary(source: &str, pos: usize, len: usize) -> bool {
    let before_ok = pos == 0
        || !source.as_bytes()[pos - 1].is_ascii_alphanumeric()
            && source.as_bytes()[pos - 1] != b'_';
    let after = pos + len;
    let after_ok = after >= source.len()
        || !source.as_bytes()[after].is_ascii_alphanumeric() && source.as_bytes()[after] != b'_';
    before_ok && after_ok
}

fn find_matching(source: &str, open: usize, open_ch: u8, close_ch: u8) -> anyhow::Result<usize> {
    let mut scanner = Scanner::new(source, open, source.len());
    let mut depth = 0usize;
    while let Some(pos) = scanner.next_code_byte() {
        match source.as_bytes()[pos] {
            ch if ch == open_ch => depth += 1,
            ch if ch == close_ch => {
                depth -= 1;
                if depth == 0 {
                    return Ok(pos);
                }
            }
            _ => {}
        }
    }
    bail!("Could not find matching delimiter in order.ncl");
}

struct Scanner<'a> {
    source: &'a str,
    offset: usize,
    end: usize,
    in_string: bool,
    escaped: bool,
    in_comment: bool,
}

impl<'a> Scanner<'a> {
    // This lightweight scanner is only for preserving common order.ncl shapes.
    // It skips quoted strings and line comments, but does not model Nickel's
    // multiline strings or interpolation expressions.
    fn new(source: &'a str, start: usize, end: usize) -> Self {
        Self {
            source,
            offset: start,
            end,
            in_string: false,
            escaped: false,
            in_comment: false,
        }
    }

    fn next_code_byte(&mut self) -> Option<usize> {
        while self.offset < self.end {
            let pos = self.offset;
            let byte = self.source.as_bytes()[pos];
            self.offset += 1;

            if self.in_comment {
                if byte == b'\n' {
                    self.in_comment = false;
                }
                continue;
            }

            if self.in_string {
                if self.escaped {
                    self.escaped = false;
                } else if byte == b'\\' {
                    self.escaped = true;
                } else if byte == b'"' {
                    self.in_string = false;
                }
                continue;
            }

            match byte {
                b'#' => {
                    self.in_comment = true;
                    continue;
                }
                b'"' => {
                    self.in_string = true;
                    continue;
                }
                _ => return Some(pos),
            }
        }
        None
    }
}
