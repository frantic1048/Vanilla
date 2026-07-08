---
name: local-tooling
description: Use when Codex needs to choose, inspect, or troubleshoot this user's local shell behavior, Vanilla dotfiles and Blend-managed config, filesystem search tools, language runtimes, package managers, PATH/toolchain issues, or platform CLIs on their macOS arm64 and Arch Linux x86_64 machines. Trigger for questions or tasks involving elvish, nushell, zsh/bash/sh availability, Vanilla dotfiles, blend view, proto, rust/cargo paths, Homebrew/pacman language tools, gh, glab, twg, pup, sentry, ripgrep, fd, or machine-specific command selection.
---

# Local Tooling

Use this skill to pick commands that fit the user's machines instead of assuming a generic Unix setup. Prefer durable conventions from the user's dotfiles, then verify current state before running version-sensitive commands.

## Quick Start

1. Treat `~/Vanilla` as the primary dotfiles checkout. It is normally present at that path across OSes.
2. Treat Vanilla configs as Blend-managed. Use `blend view` from `~/Vanilla` for read-only dotfile inspection, and do not run other Blend commands unless the user explicitly asks.
3. For environment/PATH questions, inspect `~/.local/bin/van/shellenv` and `~/.local/bin/van/shellenv.nu` first. `~/.local/bin/van` points through `~/Vanilla/bin`, which is a repo-level symlink to the physical Blend-managed script Source at `~/Vanilla/orders/bin/bin`.
4. Prefer nushell for new scripts when reasonable. Prefer elvish only for interactive-shell context or existing elvish scripts.
5. Prefer `rg` and `fd` for filesystem discovery when installed.
6. Prefer proto-managed language toolchains when possible. Avoid Homebrew/pacman language runtimes for version-sensitive work.
7. Prefer platform CLIs such as `gh`, `glab`, `twg`, `pup`, and `sentry` when installed and configured, but verify auth/config with a small read before relying on them.

## References

- Read `references/shell.md` for shell choice, dotfiles layout, and environment setup.
- Read `references/dotfiles.md` for Vanilla/Blend dotfile inspection and safety rules.
- Read `references/filesystem.md` for file and text search preferences.
- Read `references/toolchains.md` for proto, Rust, package managers, PATH, and language-runtime decisions.
- Read `references/platform-clis.md` for GitHub, GitLab, Atlassian, Datadog, and Sentry CLI handling.

## Validation

When current machine state matters, run `scripts/inspect-local-tooling.nu` if `nu` is available. Use its output as a snapshot, not as permanent truth.

If the active PATH appears to contradict this skill, do not silently build a one-off workaround. Mention the mismatch, use `~/.local/bin/van/shellenv` as the expected environment source, and ask the user if the mismatch should be fixed in dotfiles.
