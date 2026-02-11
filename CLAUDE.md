# CLAUDE.md

## Project Overview

Vanilla is a cross-platform dotfiles manager. The core tool is **blend-rs** (Rust + Nickel DSL), built from `blend-rs/` and symlinked as `./blend` at the repo root.

## Repository Layout

- `blend-rs/` — Rust source for the blend CLI
- `orders/` — Nickel-based package definitions (`.ncl` files), the active config format
- `packages/` — Legacy static config files (pre-migration)
- `screenshots/` — README screenshots
- `Brewfile*` — Homebrew dependency manifests
- `bootstrap_macos.sh`, `bootstrap_archlinux.sh` — Platform bootstrap scripts
- `NEW_BLEND.md` — Design document for the blend-rs rewrite

## Migration Status

The legacy nushell-based blend + `packages/` folder is being migrated to Rust-based `orders/` using Nickel DSL. Migration is in progress on branch `dev/new-blend`.

**Critical guardrail:** The `packages/` directory must remain untouched until migration is complete. All new configuration work goes into `orders/` as `.ncl` files.

## blend-rs Development

blend-rs is being enhanced iteratively via manual testing. No CI yet.

**Build:**
```sh
cargo build --release    # inside blend-rs/
```

**CLI commands:**
- `ship [packages...]` — Generate and deploy configs to target locations
- `view [packages...]` — Preview generated config and diff from deployed
- `sample [packages...]` — Capture deployed config as reference (reverse of ship)
- `table` — Output package info as HTML table (for README generation)
- `upgrade [step]` — System upgrade: update packages, tools, and dotfiles

**Global flags:** `--dry-run` (`-n`), `--verbose` (`-v`), `--home`, `--orders`

## Tech Stack

- Rust 1.92.0 (edition 2024, pinned in `blend-rs/rust-toolchain.toml`)
- Nickel v2 for config DSL (`nickel-lang = "2"`)
- clap v4 (derive) for CLI
- Key crates: walkdir, globset, colored, serde/serde_json, similar, rayon, anyhow
