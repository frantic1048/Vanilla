# Language Toolchains

## Proto

- Prefer proto-managed language tools when possible.
- The user syncs proto configuration from `~/Vanilla/orders/proto/.proto/.prototools` to `~/.proto/.prototools`.
- Read `~/.proto/.prototools` for the current global proto-managed versions.
- Run `proto status` or targeted version commands when current runtime versions matter.
- `~/.proto/shims`, `~/.proto/bin`, and `~/.proto/tools/node/globals/bin` should normally be on PATH through `shellenv`.

## Current Verified Proto File Shape

As of the first creation of this skill, `~/.proto/.prototools` and `~/Vanilla/orders/proto/.proto/.prototools` matched and included:

- `bun`
- `deno`
- `go`
- `proto`
- `python`
- `rust`
- `uv`
- `npm`
- `node`
- `pnpm`
- `yarn`

Do not treat these versions as permanent. Re-read the file before making version-specific claims.

## Rust

- Rust is special: executables normally live in `~/.cargo/bin`, not only under `~/.proto/shims`.
- `shellenv` should already put `~/.cargo/bin` early in PATH.
- If `command -v cargo` or `command -v rustc` points to Homebrew/pacman unexpectedly, check whether shellenv has been applied before working around it.

## Package-Manager Runtimes

- Avoid Homebrew/pacman-sourced language tools for version-sensitive work. They are harder to manage consistently and can have side effects with other package-manager packages.
- Package-manager CLIs are fine for platform utilities and OS packages, but prefer proto or repo-local pins for language runtimes and package managers.
- If PATH or runtime selection looks weird, tell the user. The user prefers fixing the shared environment source over accumulating session-specific workarounds.

## Repository-Specific Pins

Always respect repo-local version files and package-manager metadata over global defaults. Common examples include:

- `.node-version`
- `.python-version`
- `.dvmrc`
- `.prototools`
- `packageManager` in `package.json`
- Rust toolchain files

Use native tool-specific version files when present. Use `.prototools` as the fallback for tools without a native version file.
