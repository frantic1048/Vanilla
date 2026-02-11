# blend-rs: Config DSL Design

## Target Config Formats

All samples under `packages/` directory. (Ignore .nix files - legacy experimental with nix-home)

### Structured (parseable, round-trippable)

| Format | Apps | Notes |
|--------|------|-------|
| JSON | vscode settings/keybindings | jsonc (comments allowed) |
| TOML | starship, aerospace, mise, proto, alacritty, tealdeer | Most modern tools |
| YAML | pueue | Less common |

### Semi-Structured (key-value with variations)

| Format | Apps | Syntax |
|--------|------|--------|
| INI-like | git config | `[section]` + `key = value` |
| CONF | kitty, picom, ncdu, pipewire | `key value` or `key = value`, `#comments` |
| RC | various | `key value` lines |
| gitignore-like | git/ignore | lines with `#comments`, globs |

### Plaintext (no DSL transformation)

These configs have their own logic/scripting - treat as plaintext, sync via diff.

| Format | Apps | Notes |
|--------|------|-------|
| Lua | wezterm, neovim | Full scripting, `require()`, conditionals |
| Shell-like | i3wm, sway, skhd | Commands, variables, exec |
| Shells | bash, nushell, elvish | Runtime logic |
| CSS | various | Stylesheets |
| XML | fontconfig | Hierarchical with includes |

**Strategy**: Copy as-is, show plaintext diff on sync, user edits source directly.

---

## Platform-Specific Patterns Found

### 1. Comment-based conditionals (kitty)
```conf
# linux
map ctrl+shift+c copy_to_clipboard
map ctrl+shift+v paste_from_clipboard

# macos
map cmd+c copy_to_clipboard
map cmd+v paste_from_clipboard
```

### 2. Separate packages per platform
- `alacritty` vs `alacritty-macos`
- `zellij` vs `zellij-macos`

### 3. Inline comments noting arch (wezterm.lua)
```lua
-- /usr/local/bin/elvish -- x86_64
-- /opt/homebrew/bin/elvish -- aarch64
```

### 4. macOS-specific keys
```conf
macos_titlebar_color system
```

### 5. Shell env detection
Both zsh/bash source `~/bin/shellenv` for platform logic.

---

## Config Structure Patterns

### Modular includes
- fontconfig: `fonts.conf` + `conf.d/*.conf`
- pipewire: `pipewire.conf.d/`, `client.conf.d/`
- kitty: `include theme.conf`

### Plugin/version pinning
- neovim: `lazy-lock.json` (commit hashes)
- proto: `.prototools` (version pinning)
- mise: `config.toml` (tool versions)

### Schema references
```json
"$schema": "https://starship.rs/config-schema.json"
```

---

## Package Structure

Each package has two parts: **metadata** (how to sync) and **config** (what to sync).

### Format Auto-Detection

Format inferred from target file extension (can override with explicit `format`):

```rust
// Default mappings (built-in constants, extractable to config later)
const FORMAT_EXTENSIONS: &[(&str, Format)] = &[
    // Structured formats
    (".toml", Format::Toml),
    (".json", Format::Json),
    (".yaml", Format::Yaml),
    (".yml", Format::Yaml),
    (".ini", Format::Ini),
    (".conf", Format::Conf),
    (".gitconfig", Format::Ini),

    // Plaintext formats
    (".lua", Format::Plaintext),
    (".sh", Format::Plaintext),
    (".zsh", Format::Plaintext),
    (".bash", Format::Plaintext),
    (".css", Format::Plaintext),
    (".xml", Format::Plaintext),
];

// Fallback: unknown extension → Plaintext
```

### Metadata

Controls how the package is deployed:

```nickel
# packages/starship/blend.ncl
{
  blend = {
    target = "~/.config/starship.toml",  # format auto-detected as 'toml
    # format = 'toml,                    # optional: explicit override

    when = {
      os = ["darwin", "linux"],
      # arch = ["aarch64"],
      # hostname = ["chimame-tai"],
      # desktop = ["sway"],
    },
  },

  config = { ... },
}
```

### Config (structured)

For structured packages, config data is in the Nickel file:

```nickel
# packages/starship/blend.ncl
{
  blend = {
    target = "~/.config/starship.toml",
    when = { os = ["darwin", "linux"] },
  },

  config = {
    format = "$character$directory$git_branch$line_break$shell",

    character = {
      success_symbol = "[➜](bold green)",
    },

    directory = {
      truncation_length = 3,
    },
  },
}
```

This generates `~/.config/starship.toml`:
```toml
format = "$character$directory$git_branch$line_break$shell"

[character]
success_symbol = "[➜](bold green)"

[directory]
truncation_length = 3
```

### Config (plaintext)

For plaintext packages, files stored directly in package directory:

```
packages/neovim/
  blend.ncl            # metadata only (no config section)
  nvim/                # actual config files (copied as-is)
    init.lua
    lua/
      plugins.lua
```

```nickel
# packages/neovim/blend.ncl
{
  blend = {
    target = "~/.config/nvim",
    source = "nvim",           # subdirectory containing files
    # format auto-detected as 'plaintext from .lua extension

    when = {
      os = ["darwin", "linux"],
    },
  },
  # No config section - source files are copied as-is
}
```

### Conditional Values in Structured Config

Use Nickel's native pattern matching instead of embedded conditionals:

```nickel
let metadata = import "blend://metadata" in
{
  config = {
    # Simple value
    theme = "catppuccin",

    # Conditional via pattern match
    font_size = metadata.hostname |> match {
      "work-laptop" => 14,
      _ => 12,
    },

    # Platform-specific
    shell = metadata.os |> match {
      "darwin" => "/bin/zsh",
      "linux" => "/bin/bash",
      _ => "/bin/sh",
    },

    # Conditional array concatenation
    plugins =
      [ { name = "lazy.nvim" } ]
      @ (if metadata.hostname == "work-laptop"
         then [ { name = "copilot.nvim" } ]
         else []),
  },
}
```

### Field/Line Filtering

Replace git clean filters with blend metadata. Single `ignore` field, format-aware:

| Format | `ignore` interpreted as |
|--------|------------------------|
| JSON, TOML, YAML, INI | Key paths (glob: `window.zoomLevel`, `geometry.*`) |
| CONF, Plaintext | Regex patterns (`^tree_view=`, `_authToken=`) |

```nickel
# vscode (JSON) - ignore as key path
blend = {
  target = "~/.config/Code/User/settings.json",
  ignore = [
    "window.zoomLevel",
    "workbench.colorTheme",
    "geometry.*",              # glob for nested
  ],
}

# htop (CONF) - ignore as regex
blend = {
  target = "~/.config/htop/htoprc",
  ignore = [
    "^tree_view=",
    "^sort_key=",
    "_authToken=",
  ],
}
```

**Comment out (prefix with # in output)**

For lines that should be disabled but visible (plaintext only):

```nickel
blend = {
  comment_out = [
    "^proxy=",
    "^bg ",                    # sway background
  ],
}
```

This replaces git filters like `vanilla.code`, `vanilla.htop`, `vanilla.npm`.

### Metadata Fields (from kokkoro at-env)

Available in Nickel via `let metadata = import "blend://metadata" in`:

| Field | Source | Example values |
|-------|--------|----------------|
| `metadata.os` | `platform:os` | darwin, linux, windows |
| `metadata.arch` | `uname -m` | aarch64, x86_64 |
| `metadata.hostname` | `uname -n` | chimame-tai, amausaan |
| `metadata.desktop` | `$XDG_SESSION_DESKTOP` | i3, sway, gnome (or null) |
| `metadata.home` | `$HOME` | /Users/kafuuchino |
| `metadata.user` | `$USER` | kafuuchino |

---

## DSL Language Options

Avoid creating custom DSL - reuse existing languages with editor support, types, tooling.

### Comparison

| Language | Rust Integration | Types/Contracts | Turing Complete | Editor Support | Notes |
|----------|------------------|-----------------|-----------------|----------------|-------|
| **Nickel** | Native (written in Rust) | Contracts | Yes | LSP | Evolved from Nix, stable since 1.0 |
| **KCL** | Native (written in Rust) | Schema types | Yes | LSP | CNCF Sandbox, k8s focused |
| **Pkl** | Community bindings | Types | Yes | LSP | Apple, needs pkl binary |
| **CUE** | Go only | Constraints | No | LSP | Unique type=value model |
| **Dhall** | Haskell binding | Strong types | No | LSP | Most type-safe, less ergonomic |
| **Jsonnet** | C++ / Go | None | Yes | LSP | Simple, no validation |

### Recommendation: Nickel

**Pros:**
- Written in Rust - native embedding via `nickel-lang-core` crate
- Contracts (gradual typing) for validation
- JSON superset - familiar syntax
- LSP with auto-complete, type hints
- Stable since 1.0 (May 2023)
- Can serialize to JSON/YAML/TOML

**Cons:**
- Smaller ecosystem than Jsonnet/CUE
- Learning curve for contracts

### Alternative: KCL

**Pros:**
- Also written in Rust - native via `kcl-lib` crate
- Strong k8s/cloud-native ecosystem
- CNCF backing
- Multi-language bindings

**Cons:**
- More complex (designed for k8s scale)
- Overkill for dotfiles?

### Example: Nickel for blend

```nickel
# packages/starship/blend.ncl
let metadata = import "blend://metadata" in
{
  blend = {
    target = "~/.config/starship.toml",
    when = {
      os = ["darwin", "linux"],
    },
  },

  config = {
    format = "$character$directory$git_branch",

    character = {
      success_symbol = "[➜](bold green)",
    },

    # Conditional with Nickel pattern matching
    font_size = metadata.hostname |> match {
      "work-laptop" => 14,
      _ => 12,
    },

    # Platform-specific
    shell = metadata.os |> match {
      "darwin" => "/bin/zsh",
      _ => "/bin/bash",
    },
  },
}
```

### Rust Embedding (Nickel)

```rust
use nickel_lang_core::program::Program;
use nickel_lang_core::serialize;

fn eval_config(path: &str) -> Result<serde_json::Value, Error> {
    let mut program = Program::new_from_file(path)?;

    // Inject metadata (os, arch, hostname, etc.)
    program.add_import("blend://metadata", metadata_module());

    // Evaluate and serialize to JSON
    let term = program.eval_full()?;
    let json = serialize::to_json(&term)?;

    Ok(json)
}

fn metadata_module() -> NickelValue {
    // Build from kokkoro at-env or system calls
    json!({
        "os": std::env::consts::OS,      // "macos" → "darwin"
        "arch": std::env::consts::ARCH,  // "aarch64", "x86_64"
        "hostname": hostname::get()?,
        "desktop": std::env::var("XDG_SESSION_DESKTOP").ok(),
        "home": std::env::var("HOME")?,
        "user": std::env::var("USER")?,
    })
}
```

---

## DSL Requirements

Based on existing configs, the DSL needs:

### 1-3. Conditionals via Nickel Pattern Matching

Since we use Nickel, avoid raw `if/then/else` in config. Instead:
- Define `metadata` as parameter: `metadata.os`, `metadata.arch`, `metadata.hostname`, etc.
- Use `match {}` expressions for conditionals

```nickel
# packages/starship/blend.ncl
let metadata = import "blend://metadata" in

{
  blend = { ... },

  config = {
    # Platform conditional - match on os
    shell = metadata.os |> match {
      "darwin" => "/bin/zsh",
      "linux" => "/bin/bash",
      _ => "/bin/sh",
    },

    # Architecture conditional
    homebrew_prefix = metadata.arch |> match {
      "aarch64" => "/opt/homebrew",
      _ => "/usr/local",
    },

    # Hostname conditional (can use string_contains or pattern)
    font_size =
      if std.string.is_match "work" metadata.hostname then 14
      else 12,

    # Complex condition: combine multiple metadata fields
    gpu_acceleration =
      if metadata.os == "linux" && metadata.desktop == "sway" then true
      else false,
  },
}
```

**Keybindings example (array with conditionals):**

```nickel
let metadata = import "blend://metadata" in

let linux_keys = [
  { key = "ctrl+shift+c", action = "copy" },
  { key = "ctrl+shift+v", action = "paste" },
] in

let darwin_keys = [
  { key = "cmd+c", action = "copy" },
  { key = "cmd+v", action = "paste" },
] in

{
  config = {
    keybindings = metadata.os |> match {
      "darwin" => darwin_keys,
      "linux" => linux_keys,
      _ => [],
    },
  },
}
```

**Metadata object (provided by blend):**

```nickel
# blend://metadata (injected at build time)
{
  os = "darwin",           # from kokkoro platform:os
  arch = "aarch64",        # from uname -m
  hostname = "chimame-tai", # from uname -n
  desktop = null,          # from $XDG_SESSION_DESKTOP (null if unset)
  home = "/Users/kafuuchino",
  user = "kafuuchino",
}
```

This approach:
- Uses Nickel's native pattern matching (no custom DSL parsing)
- Keeps config expressions type-safe via contracts
- Allows complex conditions with boolean operators
- Editor support (LSP) works out of the box

### 4. Output format renderers needed

| Priority | Format | Complexity | Example apps |
|----------|--------|------------|--------------|
| P0 | TOML | Low | starship, aerospace, alacritty |
| P0 | JSON/JSONC | Low | vscode |
| P1 | CONF (k=v) | Low | kitty, ncdu |
| P1 | INI | Low | git config |

**Note**: Lua, shell, CSS, XML are plaintext - no renderer needed, just copy + diff.

### 5. Include/import support

Use Nickel's native import:

```nickel
let theme = import "./theme.ncl" in
let keybindings = import "./keybindings.ncl" in
{
  config = theme.config & keybindings.config & {
    # package-specific overrides
  },
}
```

### 6. Variable interpolation

Use Nickel's native string interpolation:

```nickel
let metadata = import "blend://metadata" in
let config_dir = "%{metadata.home}/.config" in
{
  config = {
    path = "%{config_dir}/nvim",
  },
}
```

---

## Sync Strategy

Since DSL contains logic, reverse sync cannot be fully automatic.

### Workflow

```
blend build          # Nickel → target files
blend diff           # show changes in target files
blend sync           # interactive: show changes, user edits source
```

### Diff Strategies by Format

| Format | Strategy | Rust Crate | Notes |
|--------|----------|------------|-------|
| JSON | Structured | `serde_json` | Lossless round-trip |
| TOML | Structured | `toml` | Good round-trip |
| YAML | Structured | `serde_yaml` | Good round-trip |
| INI | Structured | `rust-ini` | Simple sections + k=v |
| CONF | Structured | Custom parser | `key = value` or `key value` |
| Plaintext | Text diff | - | Lua, shell, etc. |

### Structured vs Text Diff

**Text Diff** (plaintext packages):
```
Generated    ←→    Current
   ↓                  ↓
 [text]            [text]
         ↓
   unified diff
```

**Structured Diff** (structured packages):
```
Generated    ←→    Current
   ↓                  ↓
 parse()           parse()
   ↓                  ↓
 Value             Value
         ↓
  semantic diff (ignores formatting, only real changes)
```

### Implementation

```rust
enum DiffStrategy {
    Structured { format: Format },
    Text,
}

enum Format {
    Json,   // serde_json
    Toml,   // toml
    Yaml,   // serde_yaml
    Ini,    // rust-ini
    Conf,   // custom: key=value or key value
}

fn diff_config(generated: &str, current: &str, strategy: DiffStrategy) -> Diff {
    match strategy {
        DiffStrategy::Structured { format } => {
            let gen_val = parse(generated, format)?;
            let cur_val = parse(current, format)?;
            semantic_diff(gen_val, cur_val)
        }
        DiffStrategy::Text => {
            unified_diff(generated, current)
        }
    }
}
```

### Simple CONF Parser

For `key = value` or `key value` formats (kitty, ncdu, etc.):

```rust
/// Lossless parse: preserves order and duplicate keys (e.g., kitty's `map`)
fn parse_conf(content: &str) -> Vec<(String, String)> {
    content.lines()
        .filter(|l| !l.trim().starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| {
            l.split_once('=')
                .or_else(|| l.split_once(' '))
                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        })
        .collect()
}

/// For diff display: group values by key while preserving key order
fn group_conf(entries: &[(String, String)]) -> IndexMap<String, Vec<String>> {
    let mut map = IndexMap::new();
    for (k, v) in entries {
        map.entry(k.clone()).or_insert_with(Vec::new).push(v.clone());
    }
    map
}
```

### Semantic Diff Output

Output format matches **source language** for easy copy-paste:

**Structured packages → Nickel expression:**
```
$ blend diff starship

# ~/.config/starship.toml changed:

# Modified:
character = {
  success_symbol = "[→](bold blue)",  # was: "[➜](bold green)"
}

# Added by app:
directory = {
  truncation_length = 5,
}

# Removed (in your source but not in deployed):
# git_branch.disabled was removed from deployed file
```

User can copy-paste Nickel syntax directly into `.ncl` source.

**Plaintext packages → unified diff:**
```
$ blend diff neovim

# ~/.config/nvim/init.lua changed:
@@ -10,3 +10,5 @@
+vim.o.wrap = true
+vim.o.linebreak = true
```

User can copy-paste into plaintext source file.

### Priority

| Priority | Format | Effort |
|----------|--------|--------|
| P0 | TOML | Low (crate) |
| P0 | JSON | Low (crate) |
| P1 | YAML | Low (crate) |
| P1 | INI | Low (crate) |
| P1 | CONF | Medium (custom) |
| P0 | Plaintext | Low (text diff) |

---

## Open Questions (Resolved)

1. ~~**Schema validation**~~ → Nickel contracts + json-schema-to-nickel
2. ~~**Expression language**~~ → Nickel native syntax
3. ~~**Sync parsing**~~ → Structured diff for JSON/TOML/YAML/INI/CONF, text diff for plaintext
4. ~~**Breaking changes**~~ → Use git worktree for migration:
   - New worktree with Nickel-based config
   - Sync logic diffs against symlinked targets
   - Replace symlinks with copied config files
5. ~~**Partial adoption**~~ → Metadata can define multiple files with different formats per package:
   ```nickel
   blend = {
     files = [
       { source = "config", target = "~/.config/app/config.toml", format = 'toml },
       { source = "theme.lua", target = "~/.config/app/theme.lua", format = 'plaintext },
     ],
   }
   ```
   Format determines sync strategy automatically:
   - `'toml`, `'json`, `'yaml`, `'ini`, `'conf` → structured diff
   - `'plaintext` → text diff
6. ~~**Nickel learning curve**~~ → Acceptable. Not heavy, sufficient features for typed config
7. ~~**Output formats**~~ → Custom renderers for INI/CONF:
   - INI: Sections + key=value (git config with filter definitions may need special escaping handling)
   - CONF: Simple key=value or key value

## Implementation Requirements

- **Rust**: Latest stable version (currently 1.84+)
- **Dependencies**: Always use latest stable versions of crates
- **Key crates**:
  - `nickel-lang-core` - Nickel evaluation
  - `serde`, `serde_json`, `toml`, `serde_yaml` - serialization
  - `indexmap` - ordered maps for CONF
  - `similar` or `diffy` - text diffing

---

## Remaining Challenges (All Resolved)

1. ~~**Git config escaping**~~ → Test carefully. If INI doesn't fit, create `Format::GitConfig` for subsections (`[remote "origin"]`) and multi-value keys.
2. ~~**Testing**~~ → Use existing configs from `packages/` as golden files. One sample per format.
3. ~~**Schema registry**~~ → Cache in `XDG_STATE_HOME/blend/schemas/` or `~/.cache/blend/schemas/`. Project-local: `.blend/schemas/`.
4. ~~**CONF multi-value keys**~~ → Use `Vec<(String, String)>` for lossless parsing (preserves order + duplicates). Group by key for diff display. `IndexMap<String, Vec<String>>` if only key-order matters.
5. ~~**Plaintext edit protection**~~ → Default to always diff before overwrite. Apps may modify deployed files; show diff so user is aware.
6. ~~**Nickel cache invalidation**~~ → Only Nickel source in repo, built files not tracked. Always rebuild from source, no stale cache.
7. ~~**Directory trees in structured packages**~~ → If `source` is a dir, auto-detect format per file. Override with explicit `{ source = "dir/file", format = ... }` in files list. Later rules override earlier (cascade).
8. ~~**JSONC round-trip**~~ → Write comments in Nickel source. Output JSON without comments is acceptable trade-off.
9. ~~**Three-way merge**~~ → `blend diff` shows deployed changes. `blend sync` opens `$EDITOR` for user to manually edit source. No auto-rewrite of Nickel expressions (conditionals can't be auto-patched). Leverage git diff for reviewing changes.
10. ~~**Secrets management**~~ → Defer to v2. Focus on core config management first.

---

## Package Inventory

59 packages total. Key ones for initial DSL support:

### DSL-managed (structured configs)

| Package | Format | Platform logic? | Priority |
|---------|--------|-----------------|----------|
| starship | TOML | No | P0 |
| aerospace | TOML | No | P0 |
| vscode | JSON | No | P0 |
| alacritty | TOML | Yes (separate pkg) | P1 |
| kitty | CONF | Yes (comments) | P1 |
| git | INI | No | P1 |

### Plaintext-managed (copy + diff)

| Package | Format | Notes |
|---------|--------|-------|
| wezterm | Lua | Has own scripting |
| neovim | Lua | Has own scripting |
| i3wm/sway | Shell-like | Has own scripting |
| bash/zsh | Shell | Runtime logic |

---

## Comparison with Existing Tools

Research sources: [chezmoi comparison table](https://www.chezmoi.io/comparison-table/), [dotfiles.github.io utilities](https://dotfiles.github.io/utilities/), [home-manager wiki](https://nixos.wiki/wiki/Home_Manager), [jade.fyi on Nix alternatives](https://jade.fyi/blog/use-nix-less/)

### Feature Matrix

| Aspect | GNU Stow | YADM | Chezmoi | Nix Home-Manager | **blend** |
|--------|----------|------|---------|------------------|-----------|
| **Approach** | Symlinks | Bare git | File copies | Nix derivations | DSL → copies |
| **Source of truth** | Actual files | Actual files | Templates | Nix expressions | Nickel files |
| **Templating** | ❌ | External (j2cli) | Go text/template | Nix language | Nickel native |
| **Conditionals** | ❌ | Alt files | Template logic | Nix logic | Nickel match/if |
| **Secrets** | ❌ | gpg | 1Password, etc | sops-nix | ❓ TBD |
| **Package mgmt** | ❌ | ❌ | ❌ | ✓ Full | ❌ |
| **Structured diff** | ❌ | ❌ | ❌ | ❌ | **✓** |
| **Bidirectional sync** | Symlinks | Git | Manual | Rebuild | **Diff + prompt** |
| **Format-aware** | ❌ | ❌ | ❌ | ❌ | **✓** |
| **Schema validation** | ❌ | ❌ | ❌ | Module types | Nickel contracts |
| **Repo complexity** | High (gitignore, git filters) | Medium (alt files) | Medium (.chezmoiignore, templates) | Low (Nix is source) | **Low (Nickel is source)** |
| **Learning curve** | Very low | Low | Medium | High | Medium |
| **Language** | Shell | Bash | Go | Nix | Rust |

### blend's Unique Value

1. **Format-aware structured diff**: Parse TOML/JSON/YAML to show semantic changes, not text diff
2. **Rust-native DSL**: Nickel embeds via crate, no subprocess or FFI
3. **Hybrid model**: Structured (TOML/JSON) and plaintext (Lua/shell) handled differently by same tool
4. **Gradual typing**: Optional Nickel contracts for schema validation

### Trade-offs

**vs GNU Stow**
- ✓ Portable configs (not symlink path-dependent)
- ✓ Built-in conditionals
- ✗ More complexity than simple symlinks

**vs Chezmoi**
- ✓ Structured diff (semantic, not text)
- ✓ Nickel contracts for schema
- ✗ Nickel ecosystem smaller than Go templates
- ✗ No built-in secrets management

**vs Home-Manager**
- ✓ Fast iteration (no Nix rebuild cycle)
- ✓ Works without Nix installed
- ✓ Lower learning curve
- ✗ No package management
- ✗ No ecosystem of pre-built modules

**vs YADM**
- ✓ Structured configs with types
- ✓ No external template tool dependency
- ✗ More opinionated structure
