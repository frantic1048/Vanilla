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
- `bootstrap.sh` — fresh-machine bootstrap entry point.
- `justfile` — canonical local task runner.
- `macos_config.sh` — standalone macOS defaults, separate from Blend.
- `README.md.nu` — root README generator.

## Working norms

- The default branch is `master`; normal development branches use `dev/*`.
- Use conventional commit subjects that describe the public repository change.
- Prefer `just` recipes, but inspect a recipe before assuming its effects.
- Do not commit generated deployment output or per-machine state unless
  explicitly requested.
- Preserve unrelated work in a dirty checkout.
- Treat real Targets and Blend state cautiously.
- Use `\u{xxxx}` escapes for non-ASCII Nickel codepoints when that improves
  readability, especially Nerd Font symbols.

## Blend baseline and state

Vanilla consumes whichever stable `blend` executable is available in `PATH`.
Orders CI independently pins the released GHCR image by version and digest.

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
just check          # validate Orders without writing them
just readme         # regenerate README.md from `blend table`
just deploy         # reconcile all Orders interactively
just sync <orders>  # reconcile selected Orders interactively
```

`just bootstrap` deploys Orders with the `blend` executable already available
in `PATH`. It is a mutating fresh-machine workflow, not a routine validation
command.

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
