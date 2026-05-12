# blend

Implementation reference for the `blend` crate. For design intent and
architectural reasoning, see [`../NEW_BLEND.md`](../NEW_BLEND.md).

## Build & test

```sh
cargo build --release      # produces target/release/blend
cargo fmt --check          # CI-equivalent format check
cargo clippy -- -D warnings
cargo test --release
```

The top-level `justfile` wraps these (`just build`, `just fmt`, `just
fmt-check`, `just clippy`, `just test`). Tests use ANSI color assertions —
run under `CLICOLOR_FORCE=1` if your terminal strips colors (CI does not).

Pinned to Rust **1.92.0** via `rust-toolchain.toml` (edition 2024).

## Source layout

```
src/
├── main.rs              entrypoint: parse CLI, build Context, dispatch
├── cli.rs               clap derive definitions
├── context.rs           runtime Context (home, orders, dry-run, metadata)
├── metadata.rs          OS/arch/hostname/desktop/user detection
├── output.rs            log helpers (info/warn/error/success)
│
├── commands.rs          re-exports cmd_sync / cmd_view / cmd_status / cmd_table
├── commands/
│   ├── helpers.rs       shared symlink + diff-aggregation helpers
│   ├── sync.rs          bidirectional sync + per-key interactive flow
│   ├── view.rs          render preview & diff
│   ├── status.rs        order state table (parallel via rayon)
│   └── table.rs         HTML table for README
│
├── compose.rs           build_order: evaluate .ncl → BuildResult
├── sync.rs              SyncMode/Action, Prompter trait, Target -> Source helpers
│
├── nickel.rs            re-exports schema types + NickelEvaluator
├── nickel/
│   ├── schema.rs        FileEntry, Format, Order, WhenCondition
│   ├── loader.rs        Nickel evaluation with metadata injection
│   ├── ast_utils.rs     parse-only shadow walk (locate_from_config)
│   └── structure_map.rs surgical .ncl rewriting via byte spans
│
├── formats.rs           FormatRenderer trait + get_renderer dispatch
├── formats/
│   ├── toml.rs          toml renderer/parser
│   ├── json.rs          JSON (with JSONC fallback on parse)
│   ├── jsonc.rs         JSON-with-comments (preserves comments on round-trip)
│   ├── delimited.rs     simple space/equals line formats
│   └── plaintext.rs     verbatim text (no parsing)
│
└── diff.rs              re-exports DiffResult, FileDiffResult, diff_*
    diff/
    ├── semantic.rs      key-based diffing (TOML/JSON/YAML)
    └── text.rs          line-based diffing for plaintext

tests/
├── sync_e2e.rs          end-to-end CLI tests (39 scenarios)
└── fixtures/            .ncl + deployed-file fixtures
```

## Where things live

| Want to change… | Look in |
| --- | --- |
| A CLI flag | `cli.rs`, then the matching `commands/<cmd>.rs` |
| How a format parses or renders | `formats/<fmt>.rs` |
| How `.ncl` evaluates | `nickel/loader.rs` |
| How `.ncl` is rewritten for Target -> Source | `nickel/structure_map.rs`, `nickel/ast_utils.rs` |
| How conflicts are presented to the user | `sync.rs` (`Prompter`, `display_conflict`) |
| Diff output for the `view` command | `commands/view.rs` + `commands/helpers.rs` |
| Status table columns | `commands/status.rs` |

## Test coverage

- **Unit tests** live alongside the code they exercise (`#[cfg(test)] mod tests`).
  Notable suites: `nickel/structure_map.rs` (AST surgery), `formats/*.rs`
  (renderer round-trips), `commands/helpers.rs` (diff aggregation).
- **End-to-end** tests in `tests/sync_e2e.rs` drive the compiled binary
  against tempdir fixtures — Source -> Target / Target -> Source sync,
  view/status flows, symlink redeploy, snapshots, and per-key interactive sync.
- The `e2e/` directory at the crate root is **legacy** (pre-migration
  stow-based fixtures) and is not wired into Cargo.

## Adding a new command

1. Add a variant to `Commands` in `cli.rs`.
2. Create `commands/<name>.rs` exporting `pub fn cmd_<name>(ctx: &Context, …) -> anyhow::Result<()>`.
3. Add `pub mod <name>;` and `pub use <name>::cmd_<name>;` to `commands.rs`.
4. Wire the dispatch arm in `main.rs`.
5. If shared with other commands, lift helpers to `commands/helpers.rs`.

## Adding a new format

1. Create `formats/<name>.rs` implementing `FormatRenderer` (`parse` →
   `serde_json::Value`, `render` → `String`).
2. Add `<Name>` to `Format` in `nickel/schema.rs` and update
   `Format::from_path` if it has a file extension.
3. Add the dispatch arm in `formats.rs::get_renderer`.
4. Add a diff strategy in `diff/semantic.rs` if structured comparison applies.
