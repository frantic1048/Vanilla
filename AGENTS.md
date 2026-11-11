# AGENTS.md

## Project overview

Vanilla is the owner's dotfiles repository. Nickel Orders under `orders/` are
deployed by the standalone [Blend](https://github.com/frantic1048/blend) CLI.
Blend implementation, product documentation, CI, and releases live in that
dedicated repository; do not add them back to Vanilla.

## Documentation authority

- `README.md.nu` generates the root `README.md` and documents Vanilla.
- `orders/order.contract.ncl` and `orders/metadata.ncl` are generated interface
  files consumed by the active Orders.
- Blend's canonical user, developer, and design documentation lives in
  `frantic1048/blend`.
- CLI help and the generated Order contract are executable interface
  references. Verify behavior against the active standalone CLI when prose and
  behavior disagree.

## Repository layout

- `orders/` — active Nickel Order definitions and literal configuration Source.
- `bin` — symlink to the personal scripts in `orders/bin/bin`.
- `legacy/` — Stow-era or out-of-scope reference material, not managed by Blend.
- `screenshots/` — root README images.
- `brewfiles/` and `Brewfile` — Homebrew dependency manifests.
- `bootstrap.sh` — fresh-machine entry point; installs nushell, then runs
  `provision.nu`.
- `provision.nu` — packages, proto toolchains, Order deployment, git credential
  helpers, and the post-provision checklist.
- `macos_config.sh` — standalone macOS defaults, separate from Blend.
- `README.md.nu` — root README generator.

## Working norms

- The default branch is `master`; normal development branches use `dev/*`.
- Commit subjects are free-form; the owner uses casual words for dotfiles
  changes. Conventional commits apply only in `frantic1048/blend`.
- Do not commit generated deployment output or per-machine state unless
  explicitly requested.
- Preserve unrelated work in a dirty checkout.
- Treat real Targets and Blend state cautiously.
- Use `\u{xxxx}` escapes for non-ASCII Nickel codepoints when that improves
  readability, especially Nerd Font symbols.

## Blend baseline and state

Vanilla installs stable `blend` from GitHub release binaries with proto, via the
TOML plugin `orders/proto/blend.toml`. The repo root `.prototools` registers it
for fresh machines (after a one-time `proto trust`); the proto Order registers it
in the global proto config. Orders CI independently pins the released GHCR
image by version and digest.

All actively managed configuration lives below `orders/`. The generated Order
contract reserves `'Absent`, `'Assert`, `'Unmanaged`, `'Unresolved`, and
`'Enforce` for resolution semantics.

Per-machine Blend state lives below `$XDG_STATE_HOME/blend/`, falling back to
`$HOME/.local/state/blend/`:

- `state.json` remembers the selected Blend Source checkout;
- `snapshots/` stores reconciliation bases.

`--blend-dir` resolution checks the nearest ancestor containing `orders/` and
then remembered state. Read commands must not refresh that state.

## Command effects

The top-level `blend` command defaults to `status`.

Read-only commands include `status`, `view`, `table`, `check`, and
`format --check`. Source-writing commands include `create`, `add`, and
`format`. `init` may write Source and configuration Targets; `sync` may write
Source, Targets, and state.

Global `--dry-run` prevents command writes, but its output is command-specific.
Force sync flags are named from Blend's perspective:
`--force-source-to-target` and `--force-target-to-source`.

`from_file` and `local` paths must be relative to their Order directory and
must not normalize outside it. File entries need an effective Target prefix
from the Order or entry.

## Common tasks

```sh
blend check          # validate Orders without writing them
nu README.md.nu      # regenerate README.md from `blend table`
blend sync           # reconcile all Orders interactively
blend sync <orders>  # reconcile selected Orders interactively
```

`./bootstrap.sh` and `provision.nu` install packages, proto, and `blend`, then
deploy Orders. They are mutating fresh-machine workflows, not routine
validation commands.

For Blend implementation changes, use a separate checkout of
`frantic1048/blend` and follow that repository's development instructions.

## CI and release history

`Orders CI` uses the pinned released `ghcr.io/frantic1048/blend` image to check
and format-check the real Order tree. Treat it as released-binary compatibility
coverage for Vanilla.

Blend versions through 0.2.16 were developed and released from Vanilla. Those
tags and GitHub Releases remain historical records. Version 0.3.0 and later
have one active source and release stream in `frantic1048/blend`; Vanilla must
not recreate Blend release workflows or tags.
