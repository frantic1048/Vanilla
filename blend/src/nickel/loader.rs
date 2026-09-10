use std::ffi::OsString;
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context as AnyhowContext, Result};
use nickel_lang::Context;

use crate::metadata::Metadata;

use super::resolution::{ResolutionNode, ResolutionPlan};
use super::schema::Order;

const BLEND_LIBRARY: &str = r#"{
  target_only = fun resolver value =>
    let selected_resolver = resolver in
    let selected_value = value in
    'BlendTargetOnly { resolver = selected_resolver, value = selected_value },
}"#;

const RESOLUTION_NORMALIZER: &str = r#"
let rec normalize = fun source target resolved =>
  if std.is_function source then
    let observed = target in
    normalize (source { target = observed }) observed true
  else if std.is_enum source then
    let enum_data = std.enum.to_tag_and_arg source in
    enum_data.tag |> match {
      "BlendTargetOnly" =>
        let payload = enum_data.arg in
        let declared = payload.value in
        if !std.is_record declared then
          std.error "blend.target_only can only be applied to a record"
        else
          let observed = target in
          let declared_children = std.record.map (fun key child =>
            let child_target =
              if std.is_record observed && std.record.has_field key observed then
                observed."%{key}"
              else
                'Absent
            in
            normalize child child_target false
          ) declared in
          let target_children =
            if std.is_record observed then
              observed
              |> std.record.filter (fun key _ => !std.record.has_field key declared)
              |> std.record.map (fun key value =>
                let child_key = key in
                let child_value = value in
                normalize
                  (payload.resolver { key = child_key, value = child_value })
                  child_value
                  true
              )
            else
              {}
          in
          { kind = "record", children = declared_children & target_children },
      "Absent" => { kind = "absent", requirement = "enforce" },
      "Unmanaged" => { kind = "unmanaged" },
      "Unresolved" => { kind = "absent", requirement = "unresolved" },
      "Enforce" =>
        if std.is_enum enum_data.arg && (std.enum.to_tag_and_arg enum_data.arg).tag == "Absent" then
          { kind = "absent", requirement = "enforce" }
        else
          { kind = "value", requirement = "enforce", value = enum_data.arg },
      "Assert" =>
        let observed = target in
        {
          kind = "assert",
          satisfied =
            if std.is_function enum_data.arg then
              enum_data.arg { target = observed }
            else
              enum_data.arg == observed,
        },
      _ =>
        let authored = source in
        {
          kind = "value",
          requirement = if resolved then "enforce" else "unresolved",
          value = authored,
        },
    }
  else if std.is_record source then
    let observed = target in
    let normalized_children = std.record.map (fun key child =>
      let child_target =
        if std.is_record observed && std.record.has_field key observed then
          observed."%{key}"
        else
          'Absent
      in
      normalize child child_target false
    ) source in
    { kind = "record", children = normalized_children }
  else
    let authored = source in
    {
      kind = "value",
      requirement = if resolved then "enforce" else "unresolved",
      value = authored,
    }
in
"#;

/// Format Nickel source using Nickel's in-process formatter.
pub fn format_source(source: &str) -> Result<String> {
    let mut output = Vec::new();
    nickel_lang_core::format::format(Cursor::new(source.as_bytes()), &mut output)
        .map_err(|e| anyhow::anyhow!("Nickel formatting error: {e}"))?;
    String::from_utf8(output).with_context(|| "Nickel formatter emitted invalid UTF-8")
}

/// Nickel evaluator with metadata injection
pub struct NickelEvaluator {
    metadata_nickel: String,
}

impl NickelEvaluator {
    /// Create a new evaluator with the given metadata
    pub fn new(metadata: &Metadata) -> Self {
        // Use Nickel record syntax (field = value), not JSON syntax (field: value),
        // because `:` means type annotation in Nickel.
        let metadata_nickel = super::ast_utils::json_to_nickel(&metadata.to_json(), 0);
        Self { metadata_nickel }
    }

    /// Evaluate an order.ncl file and return the parsed order
    pub fn evaluate(&self, ncl_path: &Path) -> Result<Order> {
        let timing = std::env::var("BLEND_TIMING").is_ok();
        let t_total = std::time::Instant::now();

        let ncl_content = std::fs::read_to_string(ncl_path)
            .with_context(|| format!("Failed to read {}", ncl_path.display()))?;

        // Inject metadata by replacing the import statement
        let processed = self.inject_metadata(&ncl_content);

        // Keep resolver functions and Blend enum sentinels lazy while loading
        // file metadata. Dynamic from_config values are evaluated later with
        // the concrete Target value for each target path.
        let program = format!(
            r#"let blend = {BLEND_LIBRARY} in
let rec has_blend_semantics = fun value =>
  if std.is_function value then true
  else if std.is_enum value then
    std.array.elem (std.enum.to_tag_and_arg value).tag
      ["BlendTargetOnly", "Absent", "Assert", "Unmanaged", "Unresolved", "Enforce"]
  else if std.is_record value then
    std.array.any (fun key => has_blend_semantics value."%{{key}}") (std.record.fields value)
  else false
in
let loaded_order = ({processed}) in
let declared_files =
  if std.record.has_field "files" loaded_order.blend then
    loaded_order.blend.files
  else
    []
in
let dynamic_entries = std.array.map (fun entry =>
  std.record.has_field "from_config" entry && has_blend_semantics entry.from_config
) declared_files in
let loaded_files = std.array.map (fun entry =>
  if std.record.has_field "from_config" entry then
    let dynamic = has_blend_semantics entry.from_config in
    entry & {{
      from_config | force = if dynamic then {{}} else entry.from_config,
    }}
  else entry
) declared_files in
{{
  blend = loaded_order.blend & {{ files | force = loaded_files }},
  __blend_dynamic_entries = dynamic_entries,
}}"#
        );

        // Evaluate the Nickel program
        let t_eval = std::time::Instant::now();
        let json = self.eval_to_json(&program, ncl_path)?;
        let eval_us = t_eval.elapsed().as_micros();

        // Parse into Order
        let mut order: Order = serde_json::from_value(json).with_context(|| {
            format!(
                "Failed to parse order.ncl structure from {}",
                ncl_path.display()
            )
        })?;

        // Resolve defaults for each file entry
        for entry in &mut order.blend.files {
            entry
                .resolve_defaults()
                .with_context(|| format!("Invalid file entry in {}", ncl_path.display()))?;
        }

        validate_order_source_paths(&order, ncl_path)?;

        if timing {
            eprintln!(
                "[timing] eval {}: total={}us nickel={}us",
                ncl_path.display(),
                t_total.elapsed().as_micros(),
                eval_us
            );
        }

        Ok(order)
    }

    /// Evaluate and normalize one dynamic `from_config` declaration against a
    /// concrete Target document. Ordinary literals remain unresolved. Scalar
    /// resolver results and explicit enforcement become automatic Source
    /// decisions; record results recurse so nested literals still fall back to
    /// interactive resolution.
    pub fn resolve_config(
        &self,
        ncl_path: &Path,
        file_entry_index: usize,
        target: Option<&serde_json::Value>,
    ) -> Result<ResolutionPlan> {
        let node = self.evaluate_resolution(ncl_path, file_entry_index, target)?;
        ResolutionPlan::from_node(node, target)
    }

    /// Exercise a dynamic declaration against the required missing-target
    /// state and a concrete present-value probe without enforcing assertions.
    /// This catches baseline resolver evaluation errors during `blend check`
    /// while leaving real target-state validation to `status`, `view`, and
    /// `sync`.
    pub fn validate_config(&self, ncl_path: &Path, file_entry_index: usize) -> Result<()> {
        let missing = self.evaluate_resolution(ncl_path, file_entry_index, None)?;
        if let Some(present_probe) = missing.into_validation_probe() {
            self.evaluate_resolution(ncl_path, file_entry_index, Some(&present_probe))?;
        }
        Ok(())
    }

    fn evaluate_resolution(
        &self,
        ncl_path: &Path,
        file_entry_index: usize,
        target: Option<&serde_json::Value>,
    ) -> Result<ResolutionNode> {
        let ncl_content = std::fs::read_to_string(ncl_path)
            .with_context(|| format!("Failed to read {}", ncl_path.display()))?;
        let processed = self.inject_metadata(&ncl_content);
        let target_nickel = target
            .map(|value| super::ast_utils::json_to_nickel(value, 0))
            .unwrap_or_else(|| "'Absent".to_string());
        let program = format!(
            r#"let blend = {BLEND_LIBRARY} in
{RESOLUTION_NORMALIZER}
let loaded_order = ({processed}) in
let declaration = (std.array.at {file_entry_index} loaded_order.blend.files).from_config in
normalize declaration ({target_nickel}) false"#
        );
        let json = match self.eval_to_json(&program, ncl_path) {
            Ok(json) => json,
            Err(error) => {
                let debug = format!("{error:#?}");
                if debug.contains("NonExhaustiveMatch") || debug.contains("NonExhaustiveEnumMatch")
                {
                    anyhow::bail!(
                        "resolver made no decision for from_config entry {file_entry_index}; \
                         Nickel's public embedding API cannot safely distinguish a direct partial \
                         match from a nested evaluation failure, so add an explicit fallback branch \
                         such as `_ => 'Unresolved`"
                    );
                }
                return Err(error).with_context(|| {
                    format!("Failed to resolve from_config entry {file_entry_index}")
                });
            }
        };
        serde_json::from_value(json).with_context(|| "Failed to decode Blend resolution result")
    }

    /// Inject runtime metadata by wrapping the `../metadata.ncl` import in a
    /// Nickel `&` merge. The committed `orders/metadata.ncl` carries
    /// `| default` annotations, so explicit values from blend override them.
    ///
    /// Limitation: matches the canonical form `import "../metadata.ncl"` only.
    /// Variants with extra whitespace or a different relative path are not
    /// rewritten. The committed orders/ files use the canonical form, and
    /// `blend init`/`sync` regenerates them deterministically.
    fn inject_metadata(&self, source: &str) -> String {
        let pattern = r#"import "../metadata.ncl""#;
        let replacement = format!(r#"((import "../metadata.ncl") & {})"#, self.metadata_nickel);
        source.replace(pattern, &replacement)
    }

    /// Evaluate Nickel source and return JSON
    fn eval_to_json(&self, source: &str, path: &Path) -> Result<serde_json::Value> {
        let mut ctx = Context::new().with_source_name(path.to_string_lossy().into_owned());

        // Add the parent directory to import paths so relative imports work
        if let Some(parent) = path.parent() {
            let import_paths: Vec<OsString> = vec![parent.as_os_str().to_owned()];
            ctx = ctx.with_added_import_paths(import_paths);
        }

        // Evaluate the Nickel source
        let expr = ctx
            .eval_deep(source)
            .map_err(|e| anyhow::anyhow!("Nickel evaluation error: {e:?}"))?;

        // Export to JSON
        let json_str = ctx
            .expr_to_json(&expr)
            .map_err(|e| anyhow::anyhow!("Failed to export Nickel to JSON: {e:?}"))?;

        let json: serde_json::Value =
            serde_json::from_str(&json_str).with_context(|| "Failed to parse exported JSON")?;

        Ok(json)
    }
}

fn validate_order_source_paths(order: &Order, ncl_path: &Path) -> Result<()> {
    for entry in &order.blend.files {
        if let Some(from_file) = &entry.from_file {
            normalize_order_source_path(from_file).with_context(|| {
                format!("Invalid from_file '{from_file}' in {}", ncl_path.display())
            })?;
        }

        if let Some(local) = &entry.local {
            normalize_order_source_path(local)
                .with_context(|| format!("Invalid local '{local}' in {}", ncl_path.display()))?;
        }

        if entry.prefix.is_empty() && order.blend.prefix.is_empty() {
            anyhow::bail!(
                "Invalid file entry '{}' in {}: no effective prefix; set blend.prefix or entry.prefix",
                entry.name,
                ncl_path.display()
            );
        }
    }

    Ok(())
}

pub fn normalize_order_source_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    if path.is_absolute() {
        anyhow::bail!("source paths must be relative to the order directory");
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    anyhow::bail!("source path escapes the order directory");
                }
            }
            Component::Prefix(_) | Component::RootDir => {
                anyhow::bail!("source paths must be relative to the order directory");
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        anyhow::bail!("source path cannot be empty");
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nickel::key_path::KeyPath;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn test_metadata() -> Metadata {
        Metadata {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            hostname: "host".to_string(),
            desktop: None,
            home: PathBuf::from("/home/test"),
            user: "test".to_string(),
        }
    }

    #[test]
    fn test_recursive_resolution_and_target_only_policy() {
        let temp = TempDir::new().unwrap();
        let order_path = temp.path().join("order.ncl");
        std::fs::write(
            temp.path().join("order.contract.ncl"),
            crate::nickel::generated::contract_ncl(),
        )
        .unwrap();
        std::fs::write(
            &order_path,
            r#"let { Order, .. } = import "./order.contract.ncl" in
({
  blend = {
    prefix = ["/tmp/"],
    files = [{
      name = "settings.json",
      from_config = {
        theme = "dark",
        mode = 'Dark,
        forced = fun _ => 2,
        defaulted = fun { target } => if target == 'Absent then 30 else target,
        maybe = fun { target } => target |> match {
          "legacy" => "modern",
          _ => 'Unresolved,
        },
        unmanaged = 'Unmanaged,
        exact = 'Enforce { owned = true },
        removed = 'Absent,
        safe = 'Assert "yes",
      } |> blend.target_only (fun { key, value } =>
        if key == "cache" then 'Unmanaged else 'Absent
      ),
    }],
  },
} | Order)"#,
        )
        .unwrap();

        let evaluator = NickelEvaluator::new(&test_metadata());
        let order = evaluator.evaluate(&order_path).unwrap();
        assert_eq!(
            order.blend.files[0].from_config,
            Some(serde_json::json!({}))
        );

        let target = serde_json::json!({
            "theme": "light",
            "mode": "Light",
            "forced": 1,
            "maybe": "experimental",
            "unmanaged": {"runtime": true},
            "exact": {"owned": false, "extra": true},
            "cache": {"runtime": true},
            "stray": true,
            "safe": "yes"
        });
        let plan = evaluator
            .resolve_config(&order_path, 0, Some(&target))
            .unwrap();
        assert_eq!(
            plan.source,
            Some(serde_json::json!({
                "theme": "dark",
                "mode": "Dark",
                "forced": 2,
                "defaulted": 30,
                "unmanaged": {"runtime": true},
                "exact": {"owned": true},
                "cache": {"runtime": true},
                "safe": "yes"
            }))
        );
        for path in ["forced", "defaulted", "exact", "removed", "stray"] {
            assert!(plan.automatic.contains(&KeyPath::root().child(path)));
        }
        assert!(!plan.automatic.contains(&KeyPath::root().child("theme")));
        assert!(!plan.automatic.contains(&KeyPath::root().child("maybe")));
        assert!(!plan.automatic.contains(&KeyPath::root().child("unmanaged")));
        assert!(!plan.automatic.contains(&KeyPath::root().child("cache")));
    }

    #[test]
    fn test_resource_root_resolver_observes_only_target() {
        let temp = TempDir::new().unwrap();
        let order_path = temp.path().join("order.ncl");
        std::fs::write(
            temp.path().join("order.contract.ncl"),
            crate::nickel::generated::contract_ncl(),
        )
        .unwrap();
        std::fs::write(
            &order_path,
            r#"let { Order, .. } = import "./order.contract.ncl" in
({
  blend = {
    prefix = ["/tmp/"],
    files = [{
      name = "root.json",
      from_config = fun { target } => 'Enforce {
        observed = target.version,
      },
    }],
  },
} | Order)"#,
        )
        .unwrap();

        let evaluator = NickelEvaluator::new(&test_metadata());
        let plan = evaluator
            .resolve_config(
                &order_path,
                0,
                Some(&serde_json::json!({"version": 3, "extra": true})),
            )
            .unwrap();

        assert_eq!(plan.source, Some(serde_json::json!({"observed": 3})));
        assert_eq!(
            plan.automatic,
            std::collections::HashSet::from([KeyPath::root()])
        );
    }

    #[test]
    fn test_target_string_interpolation_remains_data() {
        let temp = TempDir::new().unwrap();
        let order_path = temp.path().join("order.ncl");
        std::fs::write(
            temp.path().join("order.contract.ncl"),
            crate::nickel::generated::contract_ncl(),
        )
        .unwrap();
        std::fs::write(
            &order_path,
            r#"let { Order, .. } = import "./order.contract.ncl" in
({
  blend = {
    prefix = ["/tmp/"],
    files = [{
      name = "settings.json",
      from_config = { safe = 'Assert "2" },
    }],
  },
} | Order)"#,
        )
        .unwrap();

        let error = NickelEvaluator::new(&test_metadata())
            .resolve_config(
                &order_path,
                0,
                Some(&serde_json::json!({"safe": "%{1 + 1}"})),
            )
            .unwrap_err();
        assert!(error.to_string().contains("assertion failed"));
    }

    #[test]
    fn test_validate_config_evaluates_materialized_source_as_next_target() {
        let temp = TempDir::new().unwrap();
        let order_path = temp.path().join("order.ncl");
        std::fs::write(
            temp.path().join("order.contract.ncl"),
            crate::nickel::generated::contract_ncl(),
        )
        .unwrap();
        std::fs::write(
            &order_path,
            r#"let { Order, .. } = import "./order.contract.ncl" in
({
  blend = {
    prefix = ["/tmp/"],
    files = [{
      name = "settings.json",
      from_config = fun { target } =>
        if target == 'Absent then { legacy = 'Absent }
        else if std.record.has_field "legacy" target then {}
        else 1 + true,
    }],
  },
} | Order)"#,
        )
        .unwrap();

        let evaluator = NickelEvaluator::new(&test_metadata());
        let missing = evaluator.evaluate_resolution(&order_path, 0, None).unwrap();
        let generated_source = missing.into_validation_probe().unwrap();
        assert_eq!(generated_source, serde_json::json!({}));

        let error = evaluator
            .evaluate_resolution(&order_path, 0, Some(&generated_source))
            .unwrap_err();
        assert!(error.to_string().contains("resolve from_config"));

        assert!(evaluator.validate_config(&order_path, 0).is_err());
    }

    #[test]
    fn test_validate_config_does_not_enforce_target_assertions() {
        let temp = TempDir::new().unwrap();
        let order_path = temp.path().join("order.ncl");
        std::fs::write(
            temp.path().join("order.contract.ncl"),
            crate::nickel::generated::contract_ncl(),
        )
        .unwrap();
        std::fs::write(
            &order_path,
            r#"let { Order, .. } = import "./order.contract.ncl" in
({
  blend = {
    prefix = ["/tmp/"],
    files = [{
      name = "settings.json",
      from_config = {
        seed = 1,
        guarded = 'Assert "expected",
      },
    }],
  },
} | Order)"#,
        )
        .unwrap();

        NickelEvaluator::new(&test_metadata())
            .validate_config(&order_path, 0)
            .unwrap();
    }

    #[test]
    fn test_inject_metadata_wraps_real_import_with_merge() {
        let metadata = Metadata {
            os: "darwin".to_string(),
            arch: "aarch64".to_string(),
            hostname: "myhost".to_string(),
            desktop: None,
            home: PathBuf::from("/Users/test"),
            user: "test".to_string(),
        };

        let evaluator = NickelEvaluator::new(&metadata);
        let source = r#"let metadata = import "../metadata.ncl" in { os = metadata.os }"#;
        let result = evaluator.inject_metadata(source);

        // Original import expression is preserved (now inside a wrapper).
        assert!(
            result.contains(r#"import "../metadata.ncl""#),
            "wrapped output should still contain the original import: {result}"
        );
        // Wrapped with parens + `&` + a record carrying the runtime values.
        assert!(
            result.contains("&"),
            "wrapped output should contain `&` merge: {result}"
        );
        assert!(
            result.contains(r#"os = "darwin""#),
            "wrapped output should inject runtime `os` value: {result}"
        );
    }

    #[test]
    fn test_inject_metadata_no_op_without_import() {
        let metadata = Metadata {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            hostname: "host".to_string(),
            desktop: None,
            home: PathBuf::from("/home/test"),
            user: "test".to_string(),
        };
        let evaluator = NickelEvaluator::new(&metadata);
        let source = r#"{ blend = { files = [] } }"#;
        let result = evaluator.inject_metadata(source);
        assert_eq!(
            result, source,
            "order without metadata import should be unchanged"
        );
    }

    /// Pin Nickel's `&` merge strictness. Several pieces of `surgical_rewrite_with_structure`
    /// and the shadow-walk leaf collection assume that:
    ///   - `&` merging two records with an identical leaf value is fine
    ///     (the merged value equals that value);
    ///   - `&` merging two records with the SAME path bound to DIFFERENT leaf values
    ///     is a hard error (so we never silently lose one operand's value);
    ///   - disjoint fields combine into a single record.
    ///
    /// If Nickel ever loosens this contract, our code that rewrites BOTH operands
    /// to keep them in sync may need to be revisited.
    fn eval(source: &str) -> Result<serde_json::Value> {
        let mut ctx = Context::new();
        let expr = ctx
            .eval_deep(source)
            .map_err(|e| anyhow::anyhow!("eval error: {e:?}"))?;
        let json_str = ctx
            .expr_to_json(&expr)
            .map_err(|e| anyhow::anyhow!("export error: {e:?}"))?;
        Ok(serde_json::from_str(&json_str)?)
    }

    #[test]
    fn test_nickel_merge_rejects_distinct_leaf_values() {
        let err = eval("{a = 1} & {a = 2}").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("merge") || msg.to_lowercase().contains("conflict"),
            "expected merge conflict error, got: {msg}"
        );
    }

    #[test]
    fn test_nickel_merge_accepts_identical_leaf_values() {
        let v = eval("{a = 1} & {a = 1}").unwrap();
        assert_eq!(v, serde_json::json!({"a": 1}));
    }

    #[test]
    fn test_nickel_merge_combines_disjoint_fields() {
        let v = eval("{a = 1} & {b = 2}").unwrap();
        assert_eq!(v, serde_json::json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_nickel_merge_recurses_into_subrecords() {
        // Nested records may share field NAMES so long as every leaf agrees.
        let v = eval("{x = {a = 1}} & {x = {b = 2}}").unwrap();
        assert_eq!(v, serde_json::json!({"x": {"a": 1, "b": 2}}));
    }

    #[test]
    fn test_nickel_eval_accepts_unicode_brace_escapes() {
        let v = eval(r#"{ symbol = "\u{e76f} " }"#).unwrap();
        assert_eq!(v, serde_json::json!({"symbol": "\u{e76f} "}));
    }

    #[test]
    fn test_injected_merge_overrides_defaults() {
        // Mirrors what blend evaluates at runtime for an order importing
        // `../metadata.ncl`. We inline a stand-in for the imported record
        // (with `default`-annotated fields) and verify the `&` merge picks
        // the explicit runtime values over the defaults.
        let source = r#"
            let m = (
              ({ os | default = "linux", arch | default = "x86_64" })
              & { os = "darwin", arch = "aarch64" }
            ) in
            { os_out = m.os, arch_out = m.arch }
        "#;
        let v = eval(source).unwrap();
        assert_eq!(
            v,
            serde_json::json!({"os_out": "darwin", "arch_out": "aarch64"})
        );
    }
}
