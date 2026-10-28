# Shell And Environment

## Dotfiles Source

- The user's main dotfiles repo is `frantic1048/Vanilla`.
- The local checkout is normally `~/Vanilla` on every OS.
- Current migrated config lives under `~/Vanilla/orders/`.
- `~/.local/bin/van` is expected to resolve through `~/Vanilla/bin`.
- `~/Vanilla/bin` is a repo-level symlink to `~/Vanilla/orders/bin/bin`, the physical Blend-managed script Source directory.

## Shell Choice

- Nushell is the user's main interactive and login shell. Elvish is retired: its config is gone from Vanilla, and only `recall import-elvish` and `shellenv elvish` remain as bridges for machines not yet switched.
- Per-host scripts in `~/Vanilla/orders/bin/bin` share `kokkoro.nu` there for host/desktop checks. `~/Vanilla/bin` should remain usable as the stable symlinked entrypoint.
- Prefer nushell for new local automation scripts because it has rich built-ins and useful static checking, which makes agent-authored scripts easier to debug.
- Bash, sh, and zsh availability varies across the user's macOS arm64 and Arch Linux x86_64 machines. Use them for short POSIX-compatible commands, repo-native scripts, or when a tool explicitly requires them.

## Environment Source

- Treat `~/.local/bin/van/shellenv.nu` as the environment brain.
- `~/.local/bin/van/shellenv` is a bash bootstrap wrapper that finds `nu`, then delegates to `shellenv.nu`.
- Nushell config uses `use ~/.local/bin/van/shellenv.nu; shellenv apply`.
- POSIX shells can use `~/.local/bin/van/shellenv posix`.

## PATH Expectations

The shellenv helper is expected to put user-managed paths before package-manager paths. Important early entries include:

- `~/.cargo/bin`
- `~/.local/share/pnpm`
- `~/.proto/shims`
- `~/.proto/bin`
- `~/.proto/tools/node/globals/bin`
- `~/.local/bin/van`
- `~/.local/bin`

On macOS, Homebrew paths such as `/opt/homebrew/bin` or `/usr/local/bin` are still appended for general CLI availability, but avoid relying on Homebrew language runtimes for version-sensitive work.

## Drift Handling

If `command -v` shows an unexpected package-manager runtime before the user-managed path, first check whether the command ran inside the expected shellenv. Prefer fixing `shellenv` or asking the user to fix it over repeating local PATH workarounds in every task.
