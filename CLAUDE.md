# CLAUDE.md

## Project Overview

Vanilla is a cross-platform dotfiles manager. The core tool is **blend** (Rust + Nickel DSL), built from `blend/` and symlinked into `bin/blend`.

## Repository Layout

- `blend/` — Rust source for the blend CLI
- `orders/` — Nickel-based order definitions (`.ncl` files), the active config format
- `bin/` — Personal scripts deployed to `$PATH`; also where `blend` is symlinked from `target/release/blend`
- `legacy/` — Orphaned entries from the stow era kept for reference (`atom`, `krita`, `psd`, `root`); not managed by blend
- `screenshots/` — README screenshots
- `Brewfile*` — Homebrew dependency manifests
- `bootstrap.sh` — Fresh-machine bootstrap entrypoint for installing prerequisites, deploying dotfiles, and running one-shot system setup guidance
- `justfile` — Canonical task runner (build, check, test, fmt, clippy, deploy)
- `macos_config.sh` — Standalone macOS `defaults write` script (run separately from blend)
- `NEW_BLEND.md` — Design document for the blend rewrite
- `ERGO.md` — User-journey analysis / friction-point notes for blend
- `raycast-and-finder.md` — macOS Raycast + Finder setup notes
- `README.md.nu` — Nushell script that regenerates `README.md` from `blend table` output

## Migration Status

Migration from the legacy nushell-based blend + `packages/` (stow-managed) to Rust-based `orders/` (Nickel DSL) is complete on `dev/brand-new-blend`. All macOS and Linux configs live in `orders/`. The four entries that did not migrate (`atom`, `krita`, `psd`, `root`) sit in `legacy/` — defunct app, app-data rather than config, or system-level files outside blend's scope.

## blend Development

blend is being enhanced iteratively via manual testing and CI.

**Commit conventions:** use semantic/conventional commit subjects for changes to the blend program and its release/CI tooling (for example, `feat(blend): ...`, `fix(blend): ...`, `ci: ...`, `chore(blend): ...`).

**Branch conventions:** development branches should use the `dev/*` namespace.

**Pre-release posture:** blend is pre-initial-release with one user (the repo owner). Backwards compatibility is *not* a constraint — prefer the cleanest design over compat shims, deprecation paths, or migration tooling for hypothetical external users. Migrating the user's own dotfiles in this repo is still in scope.

**Build / common tasks (via `just`):**
```sh
just build         # cargo build --release + symlink target/release/blend → bin/blend
just check         # blend check (validates all orders)
just test          # cargo test --release
just fmt-check     # rustfmt --check
just clippy        # cargo clippy -- -D warnings
just deploy        # blend sync
```
Run `cargo build --release` directly inside `blend/` if you want to skip the `bin/blend` symlink step; Cargo still writes to the workspace-level `target/` directory.

**CLI commands:**
- `sync [orders...]` / `s [orders...]` — Bidirectional sync with per-key interactive Source/Target choices for `from_config` entries (`--force-source-to-target`, `--force-target-to-source`, `--no-rewrite`)
- `view [orders...]` — Preview generated config and diff from deployed (`-c` content only, `-a` all, `-s` short — omit up-to-date entries)
- `table` — Output order info as HTML table (for README generation)
- `init` — Generate or refresh `orders/order.contract.ncl` and `orders/metadata.ncl`

**Global flags:** `--dry-run` (`-n`), `--verbose` (`-v`), `--home` (Target `~` expansion + `metadata.home`), `--blend-dir`

## Tech Stack

- Rust 1.92.0 (edition 2024, pinned in `blend/rust-toolchain.toml`)
- Nickel for config DSL (`nickel-lang` + `nickel-lang-parser` from git, tag `1.16.0`)
- clap v4 (derive) for CLI
- Key crates: walkdir, globset, console, serde/serde_json, similar, rayon, anyhow, tree-sitter/tree-sitter-nickel (CST for surgical rewrite), json-strip-comments (JSONC support)
- In `.ncl` files, use `\u{xxxx}` escape sequences for non-ASCII characters (e.g. Nerd Font icons) instead of raw unicode codepoints, for readability

## CI / Release Pipeline

- **Release workflow** (`blend-v-release.yml`) is hand-maintained (no cargo-dist dependency).
- Triggered by tags matching `blend-v*` (e.g. `blend-v0.3.0`). Tags are created by release-plz (`git_release_enable = false` — release-plz only tags, the release workflow creates the GitHub Release with artifacts).
- Builds 3 targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`.
- Archives are `tar.xz` with `blend`, `README.md`, `CHANGELOG.md`, `LICENSE`.
- Shell installer template (`install_template.sh`) lives in `.github/`. CI generates a release version with the version number and SHA256 checksums embedded, uploaded as `blend-installer.sh`.
- Install path: `~/.local/bin` (XDG-correct for user-installed binaries).
- All GitHub Actions are SHA-pinned to commit digests, with `step-security/harden-runner` for egress auditing.
- Build provenance attestations via `actions/attest-build-provenance`.
