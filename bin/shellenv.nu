#!/usr/bin/env nu

# The single source of truth for environment.
#
# Two ways this file is used:
#   1. nushell imports it natively:  use ~/.local/bin/van/shellenv.nu; shellenv apply
#   2. the bin/shellenv bash wrapper exec's it to render env for other shells:
#        nu --no-config-file shellenv.nu posix    # export VAR='...' lines (zsh/bash)
#        nu --no-config-file shellenv.nu elvish   # set-env VAR '...' lines (elvish)
#
# `compute-env` is pure: returns { vars: record, path: list<string> }, mutates nothing.
# `do-side-effects` performs the one-time imperative work, guarded by the session
# marker $env.__SHELLENV_ACTIVATED.

const NODE_HEAP = "--max-old-space-size=8192"

# Directory of this file (resolved at parse-time; `path self` cannot run later).
const SELF_DIR = (path self | path dirname)

# A path entry included only if the directory exists.
def existing [p: string]: nothing -> list<string> {
    if ($p | path exists) { [$p] } else { [] }
}

# Inherited PATH normalized to a list, whether nu parsed it as a string
# (--no-config-file) or a list (configured session).
def inherited-path []: nothing -> list<string> {
    let p = ($env.PATH? | default "")
    if (($p | describe) | str starts-with "list") { $p } else { $p | split row (char esep) }
}

# Load machine-local overrides from shellenv.local.nu sitting next to this file.
# That file must define `export def overrides []: nothing -> record` returning
# { vars: record, path: list<string> }; its path entries are prepended.
def local-overrides []: nothing -> record {
    let f = ($SELF_DIR | path join "shellenv.local.nu")
    if not ($f | path exists) { return { vars: {}, path: [] } }
    let r = (nu --no-config-file -c $"use '($f)' *; overrides | to nuon" | from nuon)
    { vars: ($r.vars? | default {}), path: ($r.path? | default []) }
}

# Pure environment computation. Returns { vars, path }.
export def compute-env []: nothing -> record {
    let os = $nu.os-info.name
    let arch = $nu.os-info.arch
    let home = $nu.home-dir

    mut vars = {
        PROTO_HOME: $"($home)/.proto"
        PNPM_HOME: $"($home)/.local/share/pnpm"
        RUSTUP_HOME: $"($home)/.rustup"
        CARGO_HOME: $"($home)/.cargo"
        CODEX_INTERNAL_ORIGINATOR_OVERRIDE: "codex_cli_rs"
        GRIT_TELEMETRY_DISABLED: "true"
        VISUAL: (if (which nvim | is-not-empty) { "nvim" } else { "nano" })
        __SHELLENV_ACTIVATED: "1"
    }

    # NODE_OPTIONS: append the heap flag once. Content-based idempotency (not the
    # session marker) so it never duplicates, regardless of inherited state.
    let node_opts = ($env.NODE_OPTIONS? | default "")
    $vars = ($vars | insert NODE_OPTIONS (
        if ($node_opts | str contains $NODE_HEAP) { $node_opts } else { $"($node_opts) ($NODE_HEAP)" | str trim }
    ))

    # gpg-derived values via read-only probes.
    if (which gpgconf | is-not-empty) {
        let sock = (^gpgconf --list-dirs agent-ssh-socket | complete)
        if $sock.exit_code == 0 and ($sock.stdout | str trim | is-not-empty) {
            $vars = ($vars | insert SSH_AUTH_SOCK ($sock.stdout | str trim))
        }
    }
    if ($env.GPG_TTY? | is-empty) {
        let t = (^tty | complete)
        if $t.exit_code == 0 and ($t.stdout | str trim | is-not-empty) {
            $vars = ($vars | insert GPG_TTY ($t.stdout | str trim))
        }
    }

    mut path = []
    if $os == "macos" {
        let brew = (if $arch == "aarch64" { "/opt/homebrew" } else { "/usr/local" })
        $vars = ($vars
            | insert GOPATH $"($home)/.go"
            | insert GOBIN $"($home)/.go/bin"
            | insert NIX_SSL_CERT_FILE "/etc/ssl/certs/ca-certificates.crt"
            | insert NIX_PROFILES $"/nix/var/nix/profiles/default ($home)/.nix-profile"
            | insert HOMEBREW_PREFIX $brew
        )
        $path = [
            $"($home)/.cargo/bin"
            $vars.PNPM_HOME
            $"($vars.PROTO_HOME)/shims"
            $"($vars.PROTO_HOME)/bin"
            $"($vars.PROTO_HOME)/tools/node/globals/bin"
            $"($home)/.local/bin/van"
            ...(existing $"($home)/.local/share/mise/shims")
            ...(existing $"($home)/.go/bin")
            ...(existing $"($home)/.rd/bin")
            ...(existing $"($home)/.npm-global/bin")
            ...(existing $"($home)/.local/share/npm/bin")
            ...(existing $"($home)/.nix-profile/bin")
            ...(existing "/nix/var/nix/profiles/default/bin")
            ...(existing "/usr/local/gnupg-2.4/bin")
            ...(existing "/usr/local/opt/tcl-tk/bin")
            ...(existing $"($home)/.local/bin")
            ...(existing $"($brew)/bin")
            ...(existing $"($brew)/sbin")
        ]
    } else {
        # linux (arch)
        $path = [
            $"($home)/.cargo/bin"
            $vars.PNPM_HOME
            $"($vars.PROTO_HOME)/shims"
            $"($vars.PROTO_HOME)/bin"
            $"($vars.PROTO_HOME)/tools/node/globals/bin"
            $"($home)/.local/bin/van"
            $"($home)/.local/bin"
            $"($home)/.nix-profile/bin"
            $"($home)/.go/bin"
            $"($home)/.gem/ruby/2.2.0/bin"
            "/usr/bin/vendor_perl"
        ]
    }

    # Append inherited PATH (system paths, /etc/paths.d via inheritance) and dedup.
    $path = ($path | append (inherited-path) | uniq)

    # Machine-local overrides (prepended; vars win over defaults).
    let local = (local-overrides)
    $vars = ($vars | merge $local.vars)
    $path = ($local.path | append $path | uniq)

    { vars: $vars, path: $path }
}

# Render as POSIX `export` lines (zsh/bash). Single quotes escaped as '\''.
def render-posix [e: record]: nothing -> string {
    mut out = []
    for kv in ($e.vars | items {|k v| {k: $k, v: $v} }) {
        let safe = ($kv.v | into string | str replace --all "'" "'\\''")
        $out = ($out | append $"export ($kv.k)='($safe)'")
    }
    let pathstr = (($e.path | str join (char esep)) | str replace --all "'" "'\\''")
    $out = ($out | append $"export PATH='($pathstr)'")
    $out | str join "\n"
}

# Render as elvish `set-env` lines. Elvish single quotes escaped by doubling ('').
def render-elvish [e: record]: nothing -> string {
    mut out = []
    for kv in ($e.vars | items {|k v| {k: $k, v: $v} }) {
        let safe = ($kv.v | into string | str replace --all "'" "''")
        $out = ($out | append $"set-env ($kv.k) '($safe)'")
    }
    let pathstr = (($e.path | str join (char esep)) | str replace --all "'" "''")
    $out = ($out | append $"set-env PATH '($pathstr)'")
    $out | str join "\n"
}

# One-time imperative work, guarded so subshells don't repeat it.
# Receives the computed record so launchctl/PATH use the new values.
export def do-side-effects [computed: record] {
    if ($env.__SHELLENV_ACTIVATED? | is-not-empty) { return }
    let os = $nu.os-info.name

    if (which gpgconf | is-not-empty) {
        let proc = (if $os == "macos" { "pgrep" } else { "pidof" })
        let running = ((^$proc gpg-agent | complete | get exit_code) == 0)
        if not $running { ^gpgconf --launch gpg-agent | complete | ignore }
        if $os == "linux" and ($computed.vars.GPG_TTY? | is-not-empty) {
            ^gpg-connect-agent updatestartuptty /bye | complete | ignore
        }
    }

    if $os == "macos" and (which launchctl | is-not-empty) {
        let pathstr = ($computed.path | str join (char esep))
        ^launchctl setenv PATH $pathstr | complete | ignore
        ^launchctl setenv PROTO_HOME $computed.vars.PROTO_HOME | complete | ignore
        if ($env.SHELL? | is-not-empty) {
            ^launchctl setenv SHELL $env.SHELL | complete | ignore
        }
    }
}

# Apply the computed environment directly to the current nushell session.
# Guarded by the session marker so it runs once: nushell evaluates env.nu twice
# per startup (env + config phases) and child shells inherit an already-correct
# PATH, so re-applying would prepend our entries again.
export def --env apply [] {
    if ($env.__SHELLENV_ACTIVATED? | is-not-empty) { return }
    let e = (compute-env)
    do-side-effects $e
    load-env $e.vars
    $env.PATH = $e.path
}

export def main [shell: string = "posix"] {
    let e = (compute-env)
    do-side-effects $e
    match $shell {
        "posix" => (render-posix $e)
        "elvish" => (render-elvish $e)
        _ => { error make {msg: $"shellenv: unknown shell '($shell)'"} }
    }
}
