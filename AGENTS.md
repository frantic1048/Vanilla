# AGENTS.md

## Project overview

Vanilla is the owner's dotfiles repository. Nickel Orders under `orders/` are
deployed by the Rust `blend` CLI in `blend/`.

The repository therefore contains two related surfaces:

- personal configuration Source, bootstrap scripts, and maintenance tools;
- the Blend program, its documentation, tests, CI, and release automation.

Keep those surfaces distinct when changing behavior or interpreting CI.

## Documentation authority

- `README.md.nu` generates the root `README.md` and documents Vanilla.
- `blend/README.md` is the Blend product entry point.
- `blend/docs/GUIDE.md` defines user-visible Order and reconciliation behavior.
- `blend/docs/DEVELOPMENT.md` describes architecture, source layout, testing,
  CI, and releases.
- `blend/docs/DESIGN.md` records durable design rationale and scope boundaries.
- `blend/CHANGELOG.md` is owned by release-plz; do not edit it manually unless
  explicitly requested.

The generated `orders/order.contract.ncl` and CLI help are executable interface
references. When prose and implementation disagree, verify against the current
schema, tests, and command behavior rather than preserving stale documentation.

## Repository layout

- `blend/` — Rust crate for the Blend CLI.
- `orders/` — active Nickel Order definitions and literal configuration Source.
- `bin` — symlink to `orders/bin/bin`; `bin/blend` points through that Source
  tree to the workspace release build.
- `legacy/` — Stow-era or out-of-scope reference material, not managed by Blend.
- `screenshots/` — root README images.
- `Brewfile*` — Homebrew dependency manifests.
- `bootstrap.sh` — fresh-machine bootstrap entry point.
- `justfile` — canonical local task runner.
- `macos_config.sh` — standalone macOS defaults, separate from Blend.
- `README.md.nu` — root README generator.

## Working norms

- The default branch is `master`; normal development branches use `dev/*`.
- Use conventional commit subjects for Blend, CI, and release changes, such as
  `feat(blend): ...`, `fix(blend): ...`, `docs(blend): ...`, or `ci: ...`.
- Prefer `just` recipes, but inspect a recipe before assuming its effects.
- Do not commit generated deployment output or per-machine state unless
  explicitly requested.
- Preserve unrelated work in a dirty checkout.
- Prefer the clean current design over compatibility scaffolding for
  hypothetical users; Blend is pre-1.0 and primarily dogfooded here.
- Treat real Targets and state cautiously even when compatibility can be broken.
- Use `\u{xxxx}` escapes for non-ASCII Nickel codepoints when that improves
  readability, especially Nerd Font symbols.

## Current Blend baseline

The workspace pins Rust 1.98.0 and uses edition 2024. Direct Nickel pins in
`blend/Cargo.toml` are currently `nickel-lang 2.2.0`,
`nickel-lang-core 0.18.0`, and `nickel-lang-parser 0.3.0`. Read the manifests
before updating copied version references.

All actively managed configuration lives below `orders/`. The current generated
Order contract is version 3 and reserves `'Absent`, `'Assert`,
`'Unmanaged`, `'Unresolved`, and `'Enforce` for resolution semantics.

Per-machine state lives below `$XDG_STATE_HOME/blend/`, falling back to
`$HOME/.local/state/blend/`:

- `state.json` remembers the selected Blend Source checkout;
- `snapshots/` stores reconciliation bases.

`--blend-dir` resolution checks the nearest ancestor containing `orders/` and
then remembered state. Read commands must not refresh that state.

## Command effects

The top-level `blend` command defaults to `status`.

Read-only commands:

- `status` — deployment summary.
- `view [orders...]` — generated content and/or Target differences.
- `table` — root README Order table.
- `check [orders...]` — validate Source definitions.
- `format --check [orders...]` — formatting validation.

Source-writing commands:

- `create <order>` — scaffold an Order.
- `add <order> <target>` — import an absolute or `~`-prefixed Target into
  Source.
- `format [orders...]` — format Order source.

Source/Target/state commands:

- `init [--upgrade]` — initialize or refresh generated modules and Blend
  configuration; breaking contract migrations require `--upgrade`.
- `sync [orders...]` — reconcile Source and Target.

Global `--dry-run` prevents command writes, but its output is command-specific;
`init --dry-run` only validates generated-file freshness and does not preview
replacement content. Force sync flags are named from Blend's
perspective: `--force-source-to-target` and
`--force-target-to-source`. `--sandbox force|prefer|never` controls sandbox
installation.

`from_file` and `local` paths must be relative to their Order directory and
must not normalize outside it. File entries need an effective Target prefix
from the Order or entry.

See `blend/docs/GUIDE.md` for the complete user contract.

## Development

Common tasks from the repository root:

```sh
just build
just check
just test
just fmt-check
just clippy
```

`just build` creates a release build and refreshes `bin/blend`. A direct
`cd blend && cargo build --release` builds without the symlink step.

Start code changes from the source map in
`blend/docs/DEVELOPMENT.md#source-map`. Tests live in
`blend/tests/sync_e2e.rs` and inline `#[cfg(test)]` modules.

Generated `orders/order.contract.ncl` and `orders/metadata.ncl` originate in
`blend/src/nickel/generated.rs`. Do not edit generated copies as the source of a
schema change.

## CI and release

`Blend CI` runs on macOS and Ubuntu and checks formatting, Clippy, tests, Order
validation, and Order formatting with the in-branch binary.

`Orders CI` uses the pinned released
`ghcr.io/frantic1048/blend:0.2.15` image. It is released-binary compatibility
coverage for the real Order tree, not a replacement for Blend CI.

Both workflows use Harden Runner with outbound allowlists.

Release-plz owns release PRs, changelog updates, and `blend-v<version>` tags.
The tag-triggered Release workflow builds three platform archives, checksums,
the installer, provenance attestations, the GHCR image, and stable Homebrew tap
updates.

Release creation uses a repository GitHub App token. Do not replace it with the
default `GITHUB_TOKEN` without revalidating release API permissions.
