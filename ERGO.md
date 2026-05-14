# blend User Journey Analysis & Improvement Opportunities

## Context

Investigating the current blend implementation (at `~/Vanilla/blend/`) to map essential user journeys and identify friction points. blend is a Rust-based dotfiles manager using Nickel DSL, managing ~55 orders across macOS and Linux.

**Current CLI commands:** default status, `sync` (alias `s`), `view`, `table`,
and `init`. The previous `ship`, `sample`, and task/upgrade CLI paths have been
removed or moved to the top-level `justfile`.

---

## Journey 1: Onboarding (fresh machine bootstrap)

### Current Flow

```
1. Install Xcode CLT manually (prerequisite for git)
2. Clone Vanilla repo:  git clone <repo> ~/Vanilla
3. Run bootstrap:       ./bootstrap.sh
   3a. Install Homebrew
   3b. brew bundle install (installs system packages, including just/elvish)
   3c. Install proto and Rust/toolchains
   3d. just bootstrap (builds bin/blend, then deploys configs)
   3e. Print remaining manual shell/git/macOS steps
```

### Friction Points

| # | Issue | Severity | Detail |
|---|-------|----------|--------|
| 1 | **Bootstrap used to call `blend install`** | Resolved | `bootstrap.sh` builds `bin/blend` before deploying configs |
| 2 | **Chicken-and-egg: Rust needed to build blend** | Resolved | `bootstrap.sh` installs proto/Rust before `just bootstrap`, so fresh clones do not depend on the dangling `bin/blend` symlink |
| 3 | **No pre-built binary** | Medium | No release artifacts, no `cargo install blend`. Every fresh machine requires a full Rust compile (~minutes) |
| 4 | **blend not on PATH initially** | Medium | Binary is at `Vanilla/target/release/blend` (symlinked to `Vanilla/bin/blend`). User's shell PATH isn't configured until blend syncs the shell configs |
| 5 | **Blend dir discovery is explicit** | Resolved | `--blend-dir` points at the managed root; `context.rs` derives `orders/` from it and errors when discovery fails |
| 6 | **No first-run guidance** | Low | Running `blend` on a clean machine shows status table with all "pending" — no hint about what to do next |

### Improvement Ideas

- Keep `bootstrap.sh` as the single fresh-machine entrypoint; avoid platform-specific compatibility wrappers
- Consider distributing a pre-built binary (GitHub releases, or a small bootstrap binary that self-compiles)
- Add a first-run message: "X orders pending. Run `blend sync` to review and deploy."
- Continue first-run polish around blend dir/XDG config discovery

---

## Journey 2: New Config (adding a new app to blend)

### Current Flow

```
1. Create dir:           mkdir orders/my-app
2. Write order.ncl:      (manually, from memory or by copying another order)
3. For plaintext:        cp ~/.config/my-app/config orders/my-app/config
4. For structured:       Manually transcribe TOML/JSON/YAML into Nickel from_config syntax
5. Preview:              blend view my-app
6. Deploy:               blend sync my-app
```

### Friction Points

| # | Issue | Severity | Detail |
|---|-------|----------|--------|
| 1 | **No scaffolding command** | High | No `blend add my-app` to create order skeleton with boilerplate order.ncl |
| 2 | **Manual config transcription for structured** | High | User must hand-convert a TOML/JSON file into Nickel `from_config = { ... }` syntax. For a 200-line starship.toml, this is painful and error-prone |
| 3 | **Must know Nickel syntax** | Medium | No inline documentation, no `blend help new-order` with examples |
| 4 | **No first-class validation command** | Low | `just check` wraps `bin/blend view --dry-run`, but there is no dedicated `blend check`/`blend lint` CLI yet |
| 5 | **Schema contract usage is implicit** | Low | User should pipe to `| Order` at end of order.ncl for editor/Nickel validation, but the workflow does not strongly suggest it; Rust deserialization still validates the evaluated shape |

### Improvement Ideas

- `blend add <name> [--from <path>]` command that:
  - Creates `orders/<name>/order.ncl` with sensible defaults
  - If `--from ~/.config/app/config.toml` is given: auto-detects format, parses the file, generates `from_config` Nickel syntax using `json_to_nickel()` (already implemented in `ast_utils.rs`)
  - For directories: creates `from_file` entry pointing to copied dir
- `blend check`: validate all order.ncl files without deploying (first-class CLI wrapper around fast Nickel eval + schema check)

---

## Journey 3: Updated Config (reflecting deployed changes back to orders)

### Current Flow

```
1. App modifies its deployed config (e.g., VS Code updates settings.json)
2. Notice:               blend         → shows ≠ in DIFF column
3. Inspect:              blend view my-app  → shows semantic diff
4. Sync:                 blend sync my-app  → interactive Source/Target/skip choices
   4a. For each changed file: see diff, choose Source -> Target / Target -> Source / skip
   4b. Target -> Source: blend surgically rewrites order.ncl (even through conditional branches)
5. Verify:               blend view my-app  → should show no changes
```

### Friction Points — Resolved

These friction points from the original analysis have been addressed by `blend sync`:

| # | Original Issue | Status | How Resolved |
|---|---------------|--------|--------------|
| 1 | No assisted reverse sync | **Resolved** | `blend sync` with interactive Target -> Source. Context-aware shadow walk handles values inside match/if branches |
| 2 | Diff output in target format, not source format | **Partially resolved** | Semantic diff shows structural changes. Rewrite analysis can recover active branch context, but full Nickel-syntax diff output is still a future improvement |
| 3 | No interactive accept/reject per change | **Resolved** | Per-file Source -> Target / Target -> Source / skip / quit prompts in interactive sync |
| 4 | Auto-patch for simple data-only orders | **Resolved** | Surgical `.ncl` rewrite via AST byte spans. Works for plain data and conditional branches resolving to literals |

### Remaining Friction Points

| # | Issue | Severity | Detail |
|---|-------|----------|--------|
| 1 | **~~Per-field granularity~~** | ~~Medium~~ | **Resolved** — Per-key interactive sync for `from_config` entries: `[s]ource [t]arget s[k]ip [a]ll-source a[l]l-target [q]uit` per changed key |
| 2 | **Discovering which fields to `ignore`** | Low | Fields that apps frequently auto-update (zoom levels, timestamps) cause noisy diffs. Finding which to ignore is trial-and-error |
| 3 | **No watch/auto-detect mode** | Low | Can't monitor deployed configs for changes and notify/prompt. Must manually check `blend` status |
| 4 | **Non-rewritable fields info display** | Low | When `--no-rewrite` is active or a field can't be rewritten Target -> Source, the info display (branch context + Nickel snippet) is not yet fully implemented |
| 5 | **~~Surgical rewrite can't add/delete keys~~** | ~~Medium~~ | **Resolved** — tree-sitter-nickel CST provides StructureMap (record boundaries, field ranges, comma positions). `surgical_rewrite_with_structure()` now supports field insertion (at record's `}` with proper indentation) and deletion (full line removal). Flat dotted keys (e.g., `"workbench.editor.useModal"`) are handled by falling back to root record insertion with quoted key. |
| 6 | **Manual-fallback guidance is still thin** | Medium | Sync-back is now per-key and can partially apply, but non-rewritable expressions, structural edits inside conditional/non-literal merge operands, and `--no-rewrite` flows still need clearer branch context and suggested Nickel snippets |
| 7 | **~~No persistent deploy state / merge base~~** | ~~Medium~~ | **Resolved for conflict prompts** — snapshots now provide Base for 3-way Source/Target/Base display. Old-target cleanup remains future work. |

### Improvement Ideas

- ~~Surgical rewrite key insertion~~: **Implemented** via tree-sitter StructureMap
- ~~Surgical rewrite key deletion~~: **Implemented** via tree-sitter StructureMap
- Suggest ignore patterns: when a field keeps changing across consecutive syncs, suggest adding it to `ignore`
- Watch mode: monitor deployed configs, auto-run `blend sync` or notify on changes
- Make sync-back explicitly **tiered**:
  - **Automatic**: existing key value changes on rewritable leaves (current behavior)
  - **Assisted**: key additions/removals or non-rewritable expressions produce precise guidance instead of silent non-action
  - **Merge-based**: snapshot-backed Source/Target/Base prompts for structural conflicts
- Improve non-automatic sync ergonomics:
  - Classify changes as value-changed / key-added / key-removed / non-rewritable
  - Show the owning `from_config` entry, active branch context, and a suggested Nickel snippet for manual patching
  - Summarize what was automatically applied Target -> Source vs what still needs human edits
- Treat `from_config` and `from_file` as different ergonomics trade-offs:
  - `from_config` for stable, declarative, cross-platform config that benefits from Nickel logic
  - `from_file` for GUI-churned config files whose schemas drift often and where fidelity matters more than structure
- Extend the snapshot/base-state layer to record:
  - order / file entry / target path
  - rendered hash at deploy time
  - source identity for the originating order entry
  - deploy timestamp and machine identity
  - deployment mode (copied / symlinked / immutable)
  This state would unlock old-target cleanup after target changes and make sync diagnostics more explainable.

---

## Journey 4: Debugging & Recovery

### Current Flow

```
1. blend sync fails or produces wrong config
2. Check error message (Nickel eval error, IO error)
3. Run blend view to see generated output
4. Manually inspect order.ncl
5. No rollback — must manually restore from backup or git
```

### Friction Points

| # | Issue | Severity | Detail |
|---|-------|----------|--------|
| 1 | **No first-class validation-only command** | Medium | No `blend check` or `blend lint` CLI yet; the top-level `just check` currently uses `bin/blend view --dry-run` |
| 2 | **No rollback** | Medium | If a force deploy overwrites a config and breaks an app, there's no `blend rollback` or automatic backup |
| 3 | **Nickel errors can be opaque** | Low | Nickel evaluation errors include source info but can be hard to trace for contract violations |
| 4 | **No pre-sync backup** | Low | Sync overwrites in-place. A backup of the previous deployed version would help recovery |

### Improvement Ideas

- `blend check`: validate all orders without building (fast Nickel eval + schema check)
- Auto-backup before Source -> Target sync: copy previous Target file to `~/.cache/blend/backups/<order>/<file>.bak`
- `blend rollback <order>`: restore from backup

---

## Summary: Priority Improvements

### Quick Wins (low effort, high impact)
1. ~~**Fix bootstrap script**: install proto/Rust, build `bin/blend`, then deploy via `just bootstrap`~~
2. **First-run message**: When all orders are pending, show "Run `blend sync` to review and deploy"
3. **`blend check` command**: Validate all order.ncl files without deploying

### Medium Effort
4. **`blend add <name> [--from <path>]`**: Scaffold new orders with auto-import from existing deployed configs (can reuse existing `json_to_nickel()` for format conversion). This covers the "capture existing config into a new order" use case — currently there's no way to pull a config from the filesystem into a new order without manual setup.
5. **`--no-rewrite` info display**: Show branch context and Nickel snippets for manual merge
6. **Suggest ignore patterns**: Auto-detect frequently changing fields

### Larger Effort
7. ~~**Per-field interactive sync**~~: **Implemented** — per-key sync for `from_config` entries
8. **Pre-sync backups + rollback**: Safety net for force deployments
9. **Pre-built binary distribution**: GitHub releases or cargo-binstall support

---

## Implementation Status

Features that were in "Improvement Ideas" and are now implemented:

- **`blend sync`** — bidirectional sync with interactive Source/Target/skip choices (Journey 3, items 1/3/4)
- **`blend sync --force-source-to-target`** — non-interactive Source -> Target all
- **`blend sync --force-target-to-source`** — non-interactive Target -> Source all
- **Surgical .ncl rewrite** — auto-patches Nickel source for data-only and conditional values
- **Context-aware shadow walk** — follows active match/if branches using runtime metadata
- **`--no-rewrite` flag** — disables Target -> Source rewrite for review-only mode
- **`--dry-run` flag** — preview sync actions without changes
- **Semantic diffing** — format-aware structured comparison for TOML/JSON/JSONC and JSON-subset YAML
- **Per-key interactive sync** — `[s]ource [t]arget s[k]ip [a]ll-source a[l]l-target [q]uit` per changed key for `from_config` entries
- **Snapshot-backed 3-way prompts** — diffs show `<< Source`, `>> Target`, and `|| Base` when a Base snapshot exists
- **tree-sitter StructureMap** — CST-based record boundary and field range extraction enabling key insertion/deletion in `.ncl` files
- **JSONC format support** — parses JSON with comments/trailing commas (VS Code settings.json); JSON parser auto-falls back to JSONC
- **Directory file listing** — `blend view` enumerates per-file status for directory `from_file` entries; `--short` flag omits up-to-date files
- **`exclude` field** — glob patterns to skip files in `from_file` directories
- **`local` overlay** — machine-specific file overrides via local overlay directory (auto-created, gitignored)
- **`immutable` flag** — sets OS immutable flag (macOS `chflags uchg`, Linux `chattr +i`) on deployed files
- **Symlink detection** — auto-replaces stow symlinks with real files during sync; detects symlinked parent directories
- **Broken symlink handling** — `ensure_dir` removes broken symlinks blocking directory creation
- **Numeric equivalence** — `12` and `12.0` treated as equal in semantic diff

---

## Files Referenced

- `~/Vanilla/bootstrap.sh` — bootstrap script
- `~/Vanilla/blend/src/cli.rs` — CLI definition (Sync, View, Table, Init commands)
- `~/Vanilla/blend/src/main.rs` — command handlers (cmd_sync, cmd_view, cmd_status)
- `~/Vanilla/blend/src/compose.rs` — order discovery and build pipeline
- `~/Vanilla/blend/src/sync.rs` — bidirectional sync: pull_from_file, pull_from_config, Prompter trait
- `~/Vanilla/blend/src/nickel/ast_utils.rs` — shadow walk, surgical rewrite, json_to_nickel
- `~/Vanilla/blend/src/context.rs` — blend dir discovery logic
- `~/Vanilla/blend/src/nickel/schema.rs` — order.ncl schema types (Order, FileEntry)
- `~/Vanilla/NEW_BLEND.md` — architecture and design document
