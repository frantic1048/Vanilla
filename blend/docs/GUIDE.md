# User guide

## Mental model

Blend works with two views of a configuration:

- **Source** is the Blend checkout: `orders/<name>/order.ncl` plus any files
  stored beside it.
- **Target** is the deployed path read by an application, usually below `~`.
- An **Order** declares which source entries produce which targets and when they
  apply.

Running `blend` shows whether the active Orders agree with their Targets.
`blend view` explains differences without writing, and `blend sync` reconciles
them.

Each file entry uses exactly one source mode:

| Mode | Source representation | Target deployment | Target-to-Source behavior |
| --- | --- | --- | --- |
| `from_file` | A literal file or directory inside the Order | Copy or intentional symlink | Copies target content back; directories respect excludes and local overlays |
| `from_config` | Nickel values and resolver functions | Rendered configuration | Rewrites concrete declaration leaves where safe; resolver logic and non-rewritable expressions require manual editing |

Use `from_config` when structure, conditions, or semantic reconciliation are
valuable. Use `from_file` when byte-level fidelity or frequent application
rewrites matter more.

## Source layout

A Blend Source root contains an `orders/` directory:

```text
dotfiles/
└── orders/
    ├── order.contract.ncl
    ├── metadata.ncl
    └── editor/
        └── order.ncl
```

`blend init` creates or refreshes the generated contract and metadata modules.
For a new Source root it also creates the initial Blend configuration.
`blend create <order>` scaffolds an empty Order. `blend add` imports an existing
absolute or `~`-prefixed Target as `from_file` Source:

```sh
blend init
blend create editor
blend add git ~/.config/git
blend add shell --prefix ~/.config/shell ~/.config/shell/config
```

`--blend-dir` selects a Source root explicitly. Otherwise Blend searches the
nearest ancestor containing `orders/`, then the Source root remembered in its
per-machine state.

## Writing an Order

A structured Order normally imports and applies the generated `Order` contract:

```nickel
let { Order, .. } = import "../order.contract.ncl" in
let metadata = import "../metadata.ncl" in
{
  blend = {
    prefix = ["~/.config/editor/"],
    when = { os = ["darwin", "linux"] },

    files = [
      {
        name = "settings.toml",
        from_config = {
          theme = "dark",
          font_size = if metadata.os == "darwin" then 14 else 12,
        },
      },
      {
        from_file = "snippets",
        exclude = ["*.tmp"],
        local = "snippets.local",
      },
    ],
  },
} | Order
```

An Order may contain multiple file entries and may deploy one entry to multiple
prefixes.

### Order fields

| Field | Applies to | Meaning |
| --- | --- | --- |
| `files` | Order | File entries managed by the Order |
| `prefix` | Order or entry | Target prefixes; an entry-level value replaces the Order default |
| `when` | Order or entry | Optional `os`, `arch`, and `hostname` filters |
| `name` | Entry | Target name; required for `from_config` and inferred from `from_file` when omitted |
| `from_config` | Entry | Structured Nickel declaration rendered to the selected format |
| `from_file` | Entry | Relative Source file or directory copied to the Target |
| `format` | Entry | Explicit format when extension inference is insufficient |
| `symlink` | `from_file` | Deploy an intentional symlink instead of copying |
| `exclude` | `from_file` directory | Glob patterns omitted from the merged Source view |
| `local` | `from_file` directory | Machine-local overlay merged over tracked Source |
| `immutable` | Entry | Apply the platform immutable flag after deployment |
| `ignore` | Order or entry | Legacy diff filtering; new structured Orders should use resolution primitives |

`from_file` and `local` paths are relative to the Order directory. Absolute paths
and paths that normalize outside it are rejected. A file entry must obtain at
least one effective Target prefix from itself or its Order.

### Runtime metadata

The generated `metadata.ncl` provides editor-visible defaults. Blend injects
actual runtime values when evaluating an Order:

| Field | Typical values |
| --- | --- |
| `metadata.os` | `"darwin"`, `"linux"` |
| `metadata.arch` | `"aarch64"`, `"x86_64"` |
| `metadata.hostname` | Current hostname |
| `metadata.user` | Current user |
| `metadata.home` | Current home or `--home` override |
| `metadata.desktop` | Desktop session when available |

Use ordinary Nickel `if`, `match`, bindings, imports, and functions to compose
configuration. Keep the canonical `import "../metadata.ncl"` form: runtime
metadata injection deliberately recognizes that stable import shape.

## File deployment patterns

File entries can select deployment behavior independently of their content
format.

### Link a Target directly to Source

```nickel
files = [
  {
    name = "van",
    from_file = "bin",
    symlink = true,
  },
]
```

`symlink` is valid only with `from_file`. Instead of copying content, Blend
creates the Target as a symlink to the canonical Source path. Editing through
that Target therefore edits the tracked Source directly.

### Protect a deployed file

```nickel
files = [
  {
    from_file = ".bashrc",
    immutable = true,
  },
]
```

`immutable` applies the platform immutable flag after deployment. It can
protect either copied `from_file` content or rendered `from_config` content.
Blend clears the flag before replacing a managed Target and reapplies it when
requested. A failure to update the flag is reported as a warning because
permissions and filesystem support vary.

Keep these modes separate: `immutable` is not applied to symlink entries.

## Formats

When `format` is absent, Blend infers TOML, JSON, JSONC, or YAML from the target
extension and otherwise uses plaintext.

| Format name | Typical source shape | Comparison |
| --- | --- | --- |
| `toml` | JSON-compatible record/array/scalars | Semantic |
| `json` | JSON-compatible value | Semantic; target parsing may accept JSONC syntax |
| `jsonc` | JSON-compatible value | Semantic; generated output does not preserve target comments |
| `yaml` | JSON-compatible value | Semantic YAML 1.2 |
| `space_pair_lines` | Array of `[key, value]` pairs | Text |
| `space_record_lines` | Record rendered as `key value` | Semantic record |
| `equals_record_lines` | Record rendered as `key=value` | Semantic record |
| `plaintext` | String, or literal `from_file` content | Text |

### YAML contract

Blend deliberately implements a deterministic bridge between YAML 1.2 and its
JSON value model rather than general presentation-preserving YAML editing:

- YAML 1.2 Core Schema, exactly one document, and string mapping keys.
- Null, booleans, strings, arrays, objects, and finite JSON-compatible numbers.
- Anchors, aliases, and merge keys may be read and are expanded to ordinary
  values; they are never emitted.
- Unsupported tags, duplicate keys, cyclic or undefined aliases,
  multi-document streams, non-finite numbers, and unrepresentable values fail.
- Rendering uses stable block style, two-space indentation, sorted mapping
  keys, LF endings, and one final newline.
- Comments, quotes, scalar styles, directives, aliases, and input key order are
  not presentation-round-tripped.

For every supported value, Blend guarantees
`parse(render(value)) == value`. A parse failure never silently becomes an
empty configuration or no-op mutation.

## Declaration and resolution

The rendered Source is a declaration of intent. Every declared Source field
participates. Ordinary literals intentionally do not claim automatic authority:
if an ordinary literal differs from the Target, Blend leaves the difference
unresolved for interactive selection.

Declarative resolution uses a small enriched Nickel vocabulary:

| Authored form | Meaning when not already satisfied |
| --- | --- |
| Ordinary value | Unresolved; ask interactively |
| Resolver returning a scalar | Enforce that exact value |
| Resolver returning a record | Recurse through the returned structure |
| `'Enforce value` | Enforce the exact whole value |
| `'Absent` | Remove the field or resource |
| `'Assert expected` | Require the Target to match; fail without repairing it |
| `'Assert fun { target } => ...` | Require the predicate to hold |
| `'Unmanaged` | Outside Blend ownership; neither reconcile nor prompt |
| `'Unresolved` | Explicit resolver fallback to interactive/manual handling |
| `blend.with_target_only policy` | Apply a policy to Target children absent from Source |

At the normalized level the behavior is:

| Requirement | Already satisfied | Not satisfied |
| --- | --- | --- |
| Unresolved | No action | Interactive or manual resolution |
| Enforce | No action | Reconcile Target to satisfy the constraint |
| Assert | No action | Fail the Order without mutating to satisfy it |
| Unmanaged | No action | No reconciliation or interaction |

The same value can therefore carry different authority:

```nickel
# Divergence remains interactive.
theme = "dark",

# Divergence is repaired automatically.
theme = fun _ => "dark",

# Exact ownership is stated directly.
theme = 'Enforce "dark",
```

Resolver functions occupy the declaration position, so they observe
`{ target }` rather than receiving a separate fictional Source value. A
resolver branch that intentionally declines to decide must return
`'Unresolved`. A non-exhaustive `match` remains a Nickel error because Blend
cannot safely distinguish it from a nested evaluation failure.

### Recursive values and exactness

Structured resolver results are sparse recursive semantic trees:

```nickel
window =
  fun { target } => {
    decoration = false,
    family = fun { target } => ...,
  }
```

`decoration` is still an ordinary literal and therefore interactive on
divergence. The nested `family` resolver controls its own result. Use explicit
whole-node enforcement when every child and the exact structure are owned:

```nickel
window = 'Enforce {
  decoration = false,
  size = 10,
}
```

`null` is an ordinary present value. `'Absent` is the sole structural sentinel
for non-existence.

### Target-only children

A Target-only child has no Source field on which to attach a policy.
`blend.with_target_only` supplies that structural policy for records:

```nickel
editor =
  {
    theme = "dark",
  }
  |> blend.with_target_only 'Unmanaged
```

Here Blend manages `theme` while leaving application-generated children
outside its ownership. A constant policy applies to every Target-only child:

```nickel
# Remove every unexpected child.
plugins =
  {
    foo = "enabled",
  }
  |> blend.with_target_only 'Absent
```

Use a resolver when the policy depends on the child's key or value:

```nickel
plugins =
  {
    foo = "enabled",
  }
  |> blend.with_target_only (
    fun { key, value } =>
      match key {
        "legacy_plugin" => 'Absent,
        "runtime_plugin" => 'Unmanaged,
        "optional_plugin" => value,
        _ => 'Unresolved,
      }
  )
```

Returning `value` adopts and preserves the observed child as an actionable
exact value.

## Common composition patterns

### Supply a default only when missing

```nickel
timeout =
  fun { target } =>
    if target == 'Absent then 30 else target
```

An existing value is preserved; a missing field is created as `30`.

### Migrate selected values

```nickel
mode =
  fun { target } =>
    match target {
      "legacy" => "modern",
      "modern" => target,
      _ => 'Unresolved,
    }
```

Known states resolve automatically. Unknown states retain an explicit fallback.

### Assert a safety property

```nickel
auth_token = 'Assert 'Absent,

PermitRootLogin =
  'Assert fun { target } =>
    target != "yes",
```

Assertions verify without silently repairing the Target.

### Own only selected fields

```nickel
config =
  {
    cli_auth_credentials_store = 'Enforce "keyring",
  }
  |> blend.with_target_only 'Unmanaged
```

The declared field is enforced while application-owned state remains untouched.

## Inspecting and reconciling

| Command | Writes | Purpose |
| --- | --- | --- |
| `blend` / `blend status` | Nothing | Summarize Order and Target state |
| `blend view [orders...]` | Nothing | Preview generated content and differences |
| `blend check [orders...]` | Nothing | Validate contracts, evaluation, and supported resolution paths |
| `blend create <order>` | Source | Scaffold an Order |
| `blend add <order> <target>` | Source | Import a Target as literal Source |
| `blend format [orders...]` | Source | Format `order.ncl` files; `--check` is read-only |
| `blend init` | Source and config Target | Initialize or refresh generated modules and Blend configuration |
| `blend sync [orders...]` | Source, Target, and state | Reconcile selected Orders |
| `blend table` | Nothing | Emit the Vanilla Order table as HTML |

Useful global flags:

- `--dry-run` / `-n` prevents writes by mutating commands. Output is
  command-specific: `sync`, `add`, and `create` report prospective actions or
  content; `format` lists files that would change; `init` only checks
  generated-file freshness and does not preview replacement content.
- `--verbose` / `-v` reports paths and runtime metadata.
- `--home <path>` changes Target `~` expansion and `metadata.home`.
- `--blend-dir <path>` selects the Source root.
- `--sandbox force|prefer|never` controls process sandbox policy.

`blend view --content-only` shows generated content, `--all` shows content and
diff, and `--short` omits up-to-date entries.

`blend sync` prompts for Source, Target, skip, or quit. Structured differences
can be resolved per key. The force modes apply one direction without prompting:

```sh
blend sync --force-source-to-target
blend sync --force-target-to-source
```

Use `--no-rewrite` to prevent Target-to-Source Nickel edits and retain manual
merge behavior.

### Snapshots and reverse sync

After confirmed deployment, Blend stores Target snapshots and uses them as a
Base when explaining later differences:

- Base equals Target: Source changed.
- Base equals Source: Target changed.
- Source, Target, and Base all differ: a real conflict.

`from_file` entries are naturally reversible. `from_config` entries are
selectively reversible: Blend can rewrite concrete literal leaves, including
active metadata-driven branches, and can perform supported structural
insertions/deletions. Computed expressions, resolver logic, and ambiguous
structure require manual declaration edits. Blend degrades per field instead of
claiming that every generated configuration is reversible.

## Safety boundaries

Blend distinguishes filesystem node type from content. It does not follow an
unexpected Target symlink for Target-to-Source adoption. Replacing an unexpected
managed node requires an explicit Source selection or force operation, and only
that exact managed node is replaced; symlinked parent components are path
infrastructure and are preserved.

The process sandbox is a narrow hardening layer:

- Linux uses seccomp on supported architectures.
- macOS uses Seatbelt.
- It denies later process execution and socket-based communication.
- It is not a complete filesystem allowlist; normal filesystem ownership and
  Blend's path validation remain important.

`prefer` is the default sandbox mode. Use `force` when failure to install the
sandbox must abort, and `never` only when intentionally disabling it.

Per-machine state lives below `$XDG_STATE_HOME/blend/`, or
`$HOME/.local/state/blend/` when `XDG_STATE_HOME` is unset:

- `state.json` remembers the Source checkout.
- `snapshots/` stores reconciliation bases.

Read commands do not refresh remembered Source state.

## Upgrades and generated files

`orders/order.contract.ncl` and `orders/metadata.ncl` are generated files.
`blend init` or a successful sync refreshes compatible generated content.
Breaking contract migrations require:

```sh
blend init --upgrade
```

Contract version 3 reserves `'Absent`, `'Assert`, `'Unmanaged`,
`'Unresolved`, and `'Enforce` for resolution semantics. Review configurations
that previously used those enum tags as ordinary data before upgrading.

## Troubleshooting

- Run `blend check <order>` first for contract, Nickel, or resolver failures.
- Use `blend view --all <order>` to inspect both generated content and diff.
- Use `--verbose` to confirm the selected Source root, Target paths, and
  metadata.
- If Target-to-Source cannot rewrite a field, edit the declaration manually;
  `'Unresolved` never causes Blend to synthesize resolver logic.
- Treat YAML parse failures as real input errors. Repair the Target or select a
  safe Source deployment; do not assume malformed input means absence.
- Use `blend sync --dry-run` before a force operation when the affected Targets
  are not obvious.
