# Nushell Environment Config File
# https://github.com/nushell/nushell/blob/main/crates/nu-utils/src/sample_config/default_env.nu

# Environment comes from the centralized shellenv.nu (bin/shellenv.nu, deployed to ~/.local/bin/van).
use ~/.local/bin/van/shellenv.nu
shellenv apply

let ssh_prefix = if ([$env.SSH_CONNECTION? $env.SSH_CLIENT? $env.SSH_TTY?] | any { is-not-empty }) {
    $"[ssh (sys host | get hostname)] "
} else {
    ""
}

# Keep short paths whole; otherwise retain the last two directories.
def prompt-path [directory: string] {
    let display = if $directory == $nu.home-dir {
        "~"
    } else if ($directory | str starts-with $"($nu.home-dir)/") {
        $"~($directory | str substring ($nu.home-dir | str length)..)"
    } else {
        $directory
    }

    let parts = ($display | split row "/")
    if ($display | str length --chars) > 40 and ($parts | length) > 3 {
        $"($parts | first)/.../($parts | last 2 | str join '/')"
    } else {
        $display
    }
}

def styled-prompt-path [directory: string] {
    let display = (prompt-path $directory)
    let parts = ($display | split row "/")
    let parent = ($parts | drop 1 | str join "/")
    if ($parent | is-not-empty) {
        $"(ansi light_blue)($parent)/(ansi light_blue_bold)($parts | last)(ansi reset)"
    } else {
        $"(ansi light_blue)($display)(ansi reset)"
    }
}

# Tippy cut out of a milk-white pill, before the path: four one-cell
# Pyonpyon glyphs (orders/pyonpyon) sized to the terminal font's cell. The pill
# is one opaque foreground color and fills the cell height, so the navy
# background on the two middle cells shows only through the cut-out. Shown
# everywhere except the Linux console, which has no font fallback; PYONPYON=1
# or 0 overrides that. Milk keeps the badge neutral beside the blue path, the
# lime git ref, and the purple indicator.
def prompt-tippy [] {
    let enabled = ($env.PYONPYON? | default (if $env.TERM? == "linux" { "0" } else { "1" }))
    if $enabled != "1" { return "" }
    let pill = (ansi {fg: '#f4efe6'})
    let tippy = (ansi {fg: '#f4efe6', bg: '#142050'})
    $"($pill)\u{100003}($tippy)\u{100001}\u{100002}(ansi reset)($pill)\u{100004}(ansi reset) "
}

# Read a small regular file; refuse symlinks, FIFOs, devices and anything big,
# so a crafted .git can't hang or flood the prompt.
def prompt-read-small [file: string] {
    if ($file | path type) != "file" { return null }
    if (ls -D $file | first | get size) > 4kb { return null }
    open --raw $file | decode utf-8 | str trim
}

# Locate the git dir for the nearest repository above `directory`. Handles
# worktrees and submodules, whose `.git` is a file holding `gitdir: <path>`.
def prompt-git-dir [directory: string] {
    mut dir = $directory
    loop {
        let dotgit = ($dir | path join ".git")
        match ($dotgit | path type) {
            "dir" => { return $dotgit }
            "file" => {
                let line = (prompt-read-small $dotgit)
                if ($line == null) or not ($line | str starts-with "gitdir: ") { return null }
                return ($dir | path join ($line | str substring 8..))
            }
        }
        let parent = ($dir | path dirname)
        if $parent == $dir { return null }
        $dir = $parent
    }
}

# Current branch, or short sha when detached, read from HEAD as plain text.
# Never runs git: a repository's .git/config can make git execute commands,
# and the prompt runs in every directory you cd into. The name must match a
# plain allowlist, so a crafted HEAD shows nothing rather than injecting
# terminal escapes or a stripped look-alike name. Reftable
# repositories keep a stub HEAD and show nothing.
def prompt-git-branch [] {
    let git_dir = (prompt-git-dir $env.PWD)
    if $git_dir == null { return "" }
    let head = (prompt-read-small ($git_dir | path join "HEAD"))
    if $head == null { return "" }
    let name = if ($head | str starts-with "ref: refs/heads/") {
        $head | str substring 16..
    } else if ($head =~ '^([0-9a-f]{40}|[0-9a-f]{64})$') {
        $head | str substring 0..<7
    } else {
        return ""
    }
    if not ($name =~ '^[A-Za-z0-9._/+@-]+$') or $name == ".invalid" { return "" }
    let name = if ($name | str length) > 40 { $"($name | str substring 0..<39)…" } else { $name }
    # Tortoise-shell brackets in recall's muted gray, padded so a double-click
    # selects just the name. The soft lime marks git refs here and in recall.
    let muted = (ansi {fg: '#7d8590'})
    $" ($muted)\u{2997}(ansi reset) (ansi {fg: '#c3e88d'})($name)(ansi reset) ($muted)\u{2998}(ansi reset)"
}

let shell_indicator = $"(ansi magenta_bold)nu❯(ansi reset) "
$env.PROMPT_COMMAND = {||
    $"(ansi reset)($ssh_prefix)(prompt-tippy)(styled-prompt-path $env.PWD)(prompt-git-branch)\n"
}
$env.PROMPT_COMMAND_RIGHT = {|| "" }
$env.PROMPT_INDICATOR = {|| $shell_indicator }
$env.PROMPT_INDICATOR_VI_INSERT = {|| $shell_indicator }
$env.PROMPT_INDICATOR_VI_NORMAL = {|| $shell_indicator }
$env.PROMPT_MULTILINE_INDICATOR = {|| "::: " }

# If you want previously entered commands to have a different prompt from the usual one,
# you can uncomment one or more of the following lines.
# This can be useful if you have a 2-line prompt and it's taking up a lot of space
# because every command entered takes up 2 lines instead of 1. You can then uncomment
# the line below so that previously entered commands show with a single `🚀`.
$env.TRANSIENT_PROMPT_COMMAND = {|| "" }
$env.TRANSIENT_PROMPT_INDICATOR = {|| $shell_indicator }
# $env.TRANSIENT_PROMPT_INDICATOR_VI_INSERT = {|| "" }
# $env.TRANSIENT_PROMPT_INDICATOR_VI_NORMAL = {|| "" }
# $env.TRANSIENT_PROMPT_MULTILINE_INDICATOR = {|| "" }
# $env.TRANSIENT_PROMPT_COMMAND_RIGHT = {|| "" }

# Specifies how environment variables are:
# - converted from a string to a value on Nushell startup (from_string)
# - converted from a value back to a string when running external commands (to_string)
# Note: The conversions happen *after* config.nu is loaded
$env.ENV_CONVERSIONS = {
    "PATH": {
        from_string: { |s| $s | split row (char esep) | path expand --no-symlink }
        to_string: { |v| $v | path expand --no-symlink | str join (char esep) }
    }
    "Path": {
        from_string: { |s| $s | split row (char esep) | path expand --no-symlink }
        to_string: { |v| $v | path expand --no-symlink | str join (char esep) }
    }
}

# To add entries to PATH (on Windows you might use Path), you can use the following pattern:
# $env.PATH = ($env.PATH | split row (char esep) | prepend '/some/path')

# DO NOT REMOVE THESES NU_*_DIRS DEFAULTS!
# otherwise `config.nu` will not be able to use the default values

# Directories to search for scripts when calling source or use
# The default for this is $nu.default-config-dir/scripts
$env.NU_LIB_DIRS = [
    ($nu.default-config-dir | path join 'modules') # add <nushell-config-dir>/modules
]

# Directories to search for plugin binaries when calling register
# The default for this is $nu.default-config-dir/plugins
$env.NU_PLUGIN_DIRS = [
    ($nu.default-config-dir | path join 'plugins') # add <nushell-config-dir>/plugins
]
