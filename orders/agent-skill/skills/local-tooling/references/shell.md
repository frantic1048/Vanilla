# Shell And Environment

## Dotfiles Source

- The user's main dotfiles repo is `frantic1048/Vanilla`.
- The local checkout is normally `~/Vanilla` on every OS.
- Current migrated config lives under `~/Vanilla/orders/`.
- `~/.local/bin/van` is expected to resolve through `~/Vanilla/bin`.
- `~/Vanilla/bin` is a repo-level symlink to `~/Vanilla/orders/bin/bin`, the physical Blend-managed script Source directory.

## Shell Choice

- Elvish is the user's main interactive shell.
- Existing elvish scripts may still exist, especially in `~/Vanilla/orders/bin/bin` and elvish config. `~/Vanilla/bin` should remain usable as the stable symlinked entrypoint.
- Avoid writing new elvish scripts unless the task is specifically about elvish integration.
- Prefer nushell for new local automation scripts because it has rich built-ins and useful static checking, which makes agent-authored scripts easier to debug.
- Bash, sh, and zsh availability varies across the user's macOS arm64 and Arch Linux x86_64 machines. Use them for short POSIX-compatible commands, repo-native scripts, or when a tool explicitly requires them.

## Environment Source

- Treat `~/.local/bin/van/shellenv.nu` as the environment brain.
- `~/.local/bin/van/shellenv` is a bash bootstrap wrapper that finds `nu`, then delegates to `shellenv.nu`.
- Nushell config uses `use ~/.local/bin/van/shellenv.nu; shellenv apply`.
- Elvish config evaluates `~/.local/bin/van/shellenv elvish`.
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
