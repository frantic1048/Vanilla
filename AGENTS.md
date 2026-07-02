# AGENTS.md

## Project Overview

Vanilla is the owner's dotfiles repository. Configs are defined as Nickel
orders under `orders/` and deployed by the local `blend` CLI, a Rust tool in
`blend/`. The root `bin/blend` entry is a symlink to the release build at
`target/release/blend`.

This repo mixes two surfaces:

- dotfile source data (`orders/`, `bin/`, Brewfiles, bootstrap scripts)
- the dotfile manager itself (`blend/`)

Keep those surfaces distinct when changing, testing, and interpreting CI.

## Repository Layout

- `blend/` - Rust crate for the `blend` CLI.
- `orders/` - active Nickel order definitions and config source files.
- `bin/` - personal scripts deployed to `$PATH`, plus the `bin/blend` symlink.
- `legacy/` - stow-era or out-of-scope files kept for reference only; not
  managed by blend.
- `screenshots/` - README screenshots.
- `Brewfile*` - Homebrew dependency manifests.
- `bootstrap.sh` - fresh-machine bootstrap entrypoint.
- `justfile` - canonical local task runner.
- `macos_config.sh` - standalone macOS defaults script; run separately from
  blend.
- `NEW_BLEND.md` - design notes for the blend rewrite.
- `ERGO.md` - blend user-journey and ergonomics notes.
- `RETRO.md` - retrospective notes.
- `README.md.nu` - Nushell script that regenerates `README.md` from
  `blend table` output.

## Working Norms

- Default branch is `master`; development branches in this repo conventionally
  use `dev/*`.
- Use conventional commit subjects for blend program, CI, and release changes
  such as `feat(blend): ...`, `fix(blend): ...`, `ci: ...`, or
  `chore(blend): ...`.
- Do not manually edit `blend/CHANGELOG.md` unless asked. `release-plz` owns
  changelog updates and tag creation.
- Prefer `just` recipes for local workflows, but read the recipe before
  assuming what it does.
- Keep generated deployment output and per-machine state out of git unless the
  user explicitly asks otherwise.
- Use `\u{xxxx}` escapes for non-ASCII codepoints in `.ncl` files when that
  improves readability, especially Nerd Font symbols.

## Current Blend Status

The Rust/Nickel migration is the active implementation. All managed configs
live under `orders/`. The remaining `legacy/` entries are reference material
from the stow era or files outside blend's current scope.

Blend is already released and dogfooded, but the repo is still primarily for
one owner. When changing blend internals, prefer the clean current design over
compatibility scaffolding for hypothetical external users. Still be careful
with this owner's real dotfiles, deployed targets, and local Blend state.

Per-machine Blend state lives under `$XDG_STATE_HOME/blend/`, falling back to
`$HOME/.local/state/blend/`:

- `state.json` remembers the Blend Source checkout.
- `snapshots/` stores sync snapshots for three-way reconciliation.

`--blend-dir` resolution checks the nearest ancestor containing `orders/`, then
remembered state. Read commands must stay read-only: `status`, `view`, `table`,
and `check` must not refresh remembered state.

## Blend Development

Toolchain and dependency facts:

- Rust `1.92.0`, pinned by `blend/rust-toolchain.toml`.
- Rust edition `2024`.
- Nickel crates are exact crates.io pins in `blend/Cargo.toml`:
  `nickel-lang = "=2.1.0"`, `nickel-lang-core = "=0.17.0"`, and
  `nickel-lang-parser = "=0.2.0"`.
- Important crates include `clap`, `serde`, `serde_json`, `toml`,
  `json-strip-comments`, `similar`, `rayon`, `tree-sitter`, and
  `tree-sitter-nickel`.

Common tasks:

```sh
just build       # release build + update bin/blend symlink
just check       # bin/blend check
just test        # cargo test --release in blend/
just fmt-check   # cargo fmt --check in blend/
just clippy      # cargo clippy -- -D warnings in blend/
just deploy      # bin/blend sync
```

Use `cd blend && cargo build --release` when you only need the binary and do
not want the `bin/blend` symlink step. Cargo still writes to the workspace
`target/` directory.

## Blend CLI Semantics

The top-level `blend` command defaults to `status`.

Inspect commands:

- `status` - `[read]` show order deployment status.
- `view [orders...]` - `[read]` preview generated config and diffs from Target
  files. Useful flags: `--content-only`, `--all`, `--short`.
- `table` - `[read]` emit the README order table as HTML.

Maintain commands:

- `check [orders...]` - `[read]` validate Source order definitions.
- `format [orders...]` / `fmt` - `[source]` format Source order files; use
  `--check` in CI or review validation.
- `init --upgrade` - `[source, target]` initialize or refresh
  `orders/order.contract.ncl` and `orders/metadata.ncl`; `--upgrade` is
  required for breaking contract migrations.
- `sync [orders...]` / `s` - `[source, target]` reconcile Source orders and
  deployed Target files. Force flags are named from Blend's perspective:
  `--force-source-to-target` and `--force-target-to-source`.

Global flags:

- `--dry-run` / `-n` previews mutating commands.
- `--verbose` / `-v` logs paths and metadata.
- `--home` overrides Target `~` expansion and `metadata.home`.
- `--blend-dir` overrides the Blend Source root.
- `--sandbox force|prefer|never` controls the process sandbox policy.

## Source Map

For blend code changes, start here:

- CLI shape: `blend/src/cli.rs`, then the matching `blend/src/commands/*.rs`.
- Dispatch: `blend/src/main.rs`.
- Runtime paths, metadata, state update gate: `blend/src/context.rs`.
- Per-machine state and snapshots: `blend/src/state.rs`.
- Order evaluation: `blend/src/compose.rs` and `blend/src/nickel/loader.rs`.
- Nickel schema: `blend/src/nickel/schema.rs`.
- Source rewrite for Target-to-Source sync:
  `blend/src/nickel/structure_map.rs` and `blend/src/nickel/ast_utils.rs`.
- Format rendering/parsing: `blend/src/formats/*.rs`.
- Diffing: `blend/src/diff/*.rs`.
- Conflict flow: `blend/src/sync.rs`.
- Status table: `blend/src/commands/status.rs`.

Tests live in `blend/tests/sync_e2e.rs` and inline `#[cfg(test)]` modules.
The crate-level `blend/e2e/` directory is legacy and not wired into Cargo.

## CI And Release

`Blend CI` runs for changes to `blend/**`, root Cargo files, or its workflow.
It runs on macOS and Ubuntu and checks:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo run -- check`
- `cargo run -- format --check`

`Orders CI` runs for changes to `orders/**` or its workflow. It uses the pinned
published container image, currently `ghcr.io/frantic1048/blend:0.2.11`, then
runs `blend check` and `blend format --check`. Treat this job as
released-binary compatibility signaling for the current orders tree, not as a
replacement for testing the in-branch blend binary.

`release-plz` owns release PRs, changelog edits, and `blend-v{{ version }}` tag
creation. It is configured with `publish = false` and `git_release_enable =
false`: it tags only; the release workflow creates GitHub Releases.

The `Release` workflow runs on `blend-v*` tags. It builds archives for:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`

It uploads `tar.xz` archives, SHA256 files, a generated installer, build
provenance attestations, a GHCR Docker image, and dispatches the Homebrew tap
update for stable releases. Release creation uses a GitHub App token; do not
switch it back to the default `GITHUB_TOKEN` without revalidating release API
permissions.
