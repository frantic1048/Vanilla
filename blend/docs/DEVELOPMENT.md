# Development

This document describes the implementation and maintenance of Blend while it
lives in the Vanilla workspace. User-visible behavior belongs in
[GUIDE.md](GUIDE.md); durable design rationale belongs in
[DESIGN.md](DESIGN.md).

## Workspace and toolchain

Blend is the `blend` package in the root Cargo workspace. Cargo artifacts are
written to the root `target/` directory.

The toolchain is pinned by the root `rust-toolchain.toml`:

- Rust 1.98.0
- edition 2024

The Nickel integration uses exact direct pins in `blend/Cargo.toml`:

- `nickel-lang = 2.2.0`
- `nickel-lang-core = 0.18.0`
- `nickel-lang-parser = 0.3.0`

Other important dependencies include `clap`, `serde`, `serde_json`,
`serde-saphyr`, `toml`, `tree-sitter`, `tree-sitter-nickel`, `similar`,
`rayon`, `console`, and `rexpect`.

Prefer the manifests and lockfile over copied version statements when checking
the current dependency graph.

## Build and checks

From the repository root:

```sh
just build
just check
just test
just fmt-check
just clippy
```

The corresponding crate-level commands are:

```sh
cd blend
cargo build --release
cargo fmt --check
cargo clippy -- -D warnings
cargo test --release
cargo run -- check
cargo run -- format --check
```

`just build` also refreshes the `bin/blend` symlink. Use a direct Cargo build
when only the binary is needed.

## Architecture

At a high level:

```text
order.ncl + Source files
        │
        ▼
contract and metadata-aware Nickel loading
        │
        ├── static file-entry metadata
        └── deferred enriched from_config declarations
        │
        ▼
Target parse and recursive resolution
        │
        ▼
ResolutionPlan + rendered Source
        │
        ▼
node-type/content comparison + snapshot context
        │
        ▼
view, interactive sync, forced reconciliation, or failure
```

### Source map

| Area | Primary files |
| --- | --- |
| Process entry and command dispatch | `src/main.rs` |
| CLI definitions | `src/cli.rs` |
| Runtime paths, metadata, Source discovery | `src/context.rs`, `src/metadata.rs` |
| Per-machine state and snapshots | `src/state.rs` |
| Process sandbox | `src/sandbox.rs` |
| Order discovery, evaluation, and deployment | `src/compose.rs` |
| Interactive reconciliation and Source rewriting | `src/sync.rs` |
| Filesystem node-type comparison | `src/fs_node.rs` |
| Immutable flag handling | `src/immutable.rs` |
| Contract migration | `src/migration.rs` |
| Command handlers | `src/commands/*.rs` |
| Format dispatch and implementations | `src/formats.rs`, `src/formats/*.rs` |
| Semantic and text diffing | `src/diff.rs`, `src/diff/*.rs` |
| Nickel loading and generated modules | `src/nickel/loader.rs`, `src/nickel/generated.rs` |
| Resolution normalization result | `src/nickel/resolution.rs` |
| Order schema | `src/nickel/schema.rs` |
| Diagnostics and generated/original span mapping | `src/nickel/diagnostics.rs`, `src/nickel/source_map.rs` |
| Target-to-Source leaf and structural mapping | `src/nickel/ast_utils.rs`, `src/nickel/structure_map.rs` |
| Structured key identity | `src/nickel/key_path.rs` |

`src/commands/helpers.rs` contains shared comparison and diff-aggregation
behavior. `src/commands/status.rs` performs parallel status evaluation with
Rayon.

## Order loading and resolution

### Generated contract and metadata

`src/nickel/generated.rs` owns the generated `orders/order.contract.ncl` and
`orders/metadata.ncl` content. Read commands check their freshness but must not
repair them. `init` and `sync` may refresh compatible generated files; breaking
contract changes are applied only by `init --upgrade`.

The tracked metadata module supplies stable defaults for Nickel contracts and
language tooling. `NickelEvaluator` recognizes the canonical
`import "../metadata.ncl"` expression and merges runtime metadata over it.
`src/nickel/source_map.rs` records which generated spans still correspond to
original Order source so diagnostics point back to user code instead of wrapper
expressions.

### Deferred enriched declarations

A `from_config` value containing resolver functions or reserved semantic enum
tags cannot be exported directly as ordinary JSON. Initial Order loading
therefore evaluates file metadata while marking dynamic entries. For each
concrete Target, the evaluator re-evaluates that entry with the observed Target
value and normalizes the enriched Nickel result.

The embedded Blend library implements `blend.with_target_only` by wrapping the
record and policy in an internal sentinel. The normalizer interprets ordinary
values, resolver functions, `'Absent`, `'Assert`, `'Unmanaged`,
`'Unresolved`, and `'Enforce` recursively.

`src/nickel/resolution.rs` turns the normalized tree into a `ResolutionPlan`:

- `source` is the materialized desired structured value, if present.
- `automatic` contains authoritative key paths that may be reconciled without a
  prompt.
- `manual` contains unresolved paths requiring interaction or declaration edits.
- `resource` records root presence, absence, unmanaged, or unresolved state.

Assertions are checked while materializing the plan. An unsatisfied assertion
fails with its structural path.

Strings and field names crossing the Target observation boundary are serialized
as literal Nickel data. Malformed structured Target input is an error, not
`'Absent`.

## Rendering and parsing

Every format implements `FormatRenderer`:

```rust
trait FormatRenderer {
    fn render(&self, value: &serde_json::Value) -> Result<String>;
    fn parse(&self, content: &str) -> Result<serde_json::Value>;
}
```

Implementations live under `src/formats/`:

- `toml.rs`
- `json.rs`
- `jsonc.rs`
- `yaml.rs`
- `delimited.rs` for space/equals line formats
- `plaintext.rs`

TOML, JSON, JSONC, YAML, `space_record_lines`, and
`equals_record_lines` use structured comparison where applicable.
`space_pair_lines` and plaintext use text comparison.

YAML uses the deliberately bounded YAML 1.2/JSON conversion contract described
in the user guide. `serde-saphyr` is pinned exactly because duplicate-key,
alias, tag, numeric, and source-location behavior are part of the contract.

## Comparison and deployment

`compose::build_order` produces one or more `BuildResult` values containing the
Target path, rendered or literal Source, format, ownership paths, resource
disposition, and deployment options.

Comparison has two independent dimensions:

1. filesystem node type: missing, regular file, directory, or symlink;
2. content or symlink destination when the types are compatible.

Unexpected symlinks are never followed for ordinary Target-to-Source adoption.
Source-to-Target replacement operates on the exact managed node and leaves
ancestor symlinks alone.

Directory `from_file` entries are built from the tracked Source plus an optional
local overlay, with excludes applied to the merged view. That view—not every
file that happens to exist below the Target directory—is Blend's ownership
boundary.

Successful reconciliation writes a snapshot through `StateStore`. Snapshot
paths mirror the absolute Target below an Order-specific directory and are
written atomically via temporary file and rename.

## Target-to-Source rewriting

`from_file` reconciliation copies bytes or directory content back to Source.

`from_config` reconciliation is selective. Blend deliberately uses two Nickel
representations:

- `nickel-lang-parser` AST nodes and `TermPos` spans locate concrete values,
  including active `match` and `if` branches.
- `tree-sitter-nickel` CST nodes retain record boundaries, field ranges, commas,
  indentation, and attached comments for structural insertion/deletion.

`KeyPath` stores literal path segments so field names containing dots or
brackets remain unambiguous. Edits are applied from the end of the source
towards the beginning to preserve byte offsets.

The resolution plan retains whether an unresolved node maps to a concrete
editable declaration. Blend may rewrite such a value after a Target selection,
but it never invents edits to resolver logic. Partial pulls report both applied
and unapplied paths.

## State and sandbox

`StateStore` resolves its root from `XDG_STATE_HOME`, falling back to
`$HOME/.local/state`. It owns:

- `blend/state.json` for the remembered Source checkout;
- `blend/snapshots/` for Source/Target/Base reconciliation context.

Only commands allowed to mutate state may update the remembered checkout.
`status`, `view`, `table`, and `check` remain read-only.

The first sandbox layer is intentionally process-scoped:

- Linux installs a seccomp filter on supported architectures.
- macOS installs a Seatbelt profile.
- Later process execution and socket-based communication are denied.
- Filesystem access is still governed by ordinary permissions, validated Source
  paths, evaluated Target paths, and command behavior.

The default `prefer` policy continues with a warning if sandbox installation is
unavailable; `force` makes failure fatal and `never` disables installation.

## Tests

Unit tests live beside the code they exercise. Important concentrated suites
cover:

- Nickel evaluation, normalization, source mapping, and diagnostics;
- AST/CST source surgery and attached-comment behavior;
- renderer/parser contracts, including YAML edge cases;
- semantic and text diffs;
- state paths and atomic snapshot writes;
- node-type and sandbox behavior.

`tests/sync_e2e.rs` drives the compiled CLI against temporary Source and Target
trees. It covers initialization, add/create/check/view/status, both sync
directions, interactive per-key decisions, resolution primitives, snapshots,
symlinks, directory overlays, contract migration, and diagnostics.

When changing a public semantic, add both focused normalization coverage and an
end-to-end case showing its observable sync behavior.

## Adding a command

1. Add the Clap shape in `src/cli.rs`.
2. Add `src/commands/<name>.rs` and export it from `src/commands.rs`.
3. Wire dispatch in `src/main.rs`.
4. Put genuinely shared command behavior in `src/commands/helpers.rs`.
5. Add CLI-level tests for read/write classification and failure behavior.

## Adding a format

1. Add the `Format` variant and extension inference in
   `src/nickel/schema.rs`.
2. Implement `FormatRenderer` under `src/formats/`.
3. Add the dispatcher arm in `src/formats.rs`.
4. Select semantic or text comparison explicitly.
5. Define the accepted value model, deterministic output, parse failures, and
   round-trip guarantee.
6. Add unit fixtures and end-to-end Order behavior.

## CI

`Blend CI` runs on macOS and Ubuntu for Blend/workspace changes:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo run -- check`
- `cargo run -- format --check`

`Orders CI` uses the pinned released
`ghcr.io/frantic1048/blend:0.2.15` image to validate and format-check the real
`orders/` tree. It is released-binary compatibility coverage, not a substitute
for testing the in-branch Blend implementation.

Both blocking CI workflows use Harden Runner with an outbound allowlist.

## Release flow

`release-plz` owns release PRs, `blend/CHANGELOG.md`, and
`blend-v<version>` tags. The workspace is configured for Git-only releases:
Cargo publication and release creation are disabled in release-plz itself.

A `blend-v*` tag starts the Release workflow, which:

1. builds archives for Apple aarch64, Apple x86_64, and Linux x86_64;
2. emits SHA256 files and build-provenance attestations;
3. generates the installer and creates the GitHub Release;
4. publishes the GHCR image with provenance;
5. dispatches stable versions to the Homebrew tap.

Prereleases do not receive the container `latest` tag or a Homebrew update.
GitHub Releases use a repository GitHub App token; changing that authentication
requires revalidating release permissions.
