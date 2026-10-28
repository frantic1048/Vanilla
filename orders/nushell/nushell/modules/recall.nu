#!/usr/bin/env nu

# Elvish-style command and directory history pickers backed by SQLite.
#
# Commands come from Nushell's own SQLite history. Directories live in
# dirs.sqlite3 next to it and are scored like Elvish's directory history:
# every visit decays all scores and then bumps the visited directory.
# ref: https://github.com/elves/elvish/blob/v0.21.0/pkg/store/dir.go

const dir_score_decay = 0.986 # roughly 0.5^(1/50)
const dir_score_increment = 10

def dirs-db-path [] {
    $nu.history-path | path dirname | path join dirs.sqlite3
}

def open-dirs-db [] {
    let db = (dirs-db-path)
    if not ($db | path exists) {
        # `open` needs an existing SQLite file; seed one, then define the real schema.
        mkdir ($db | path dirname)
        [{init: true}] | into sqlite $db -t init
        open $db | query db "CREATE TABLE IF NOT EXISTS dirs (path TEXT PRIMARY KEY, score REAL NOT NULL)"
        open $db | query db "DROP TABLE IF EXISTS init"
        open $db | query db "PRAGMA journal_mode = WAL" | ignore
    }
    open $db
}

def tilde-abbr [] {
    let dir = $in
    if $dir == $nu.home-dir {
        "~"
    } else if ($dir | str starts-with $"($nu.home-dir)/") {
        $"~($dir | str substring ($nu.home-dir | str length)..)"
    } else {
        $dir
    }
}

# Unique commands, most recent first.
def recent-commands [] {
    open $nu.history-path
    | query db "SELECT command_line FROM history GROUP BY command_line ORDER BY max(id) DESC"
    | get command_line
}

def scored-dirs [] {
    open-dirs-db | query db "SELECT path, score FROM dirs ORDER BY score DESC"
}

# Fuzzy picker with an Elvish-like mode line title. The color styles the title,
# matched characters, and the focus marker; config changes stay in this scope.
def pick [title: string, color: string, --multi, --display: closure] {
    let items = $in # read before assigning $env.config, which drops the input
    $env.config.color_config.search_result = {fg: $color, attr: bu}
    $env.config.color_config.row_index = {fg: $color, attr: b}
    let prompt = $"(ansi {fg: $color, attr: rb}) ($title) (ansi reset)"
    if $display == null {
        $items | input list --fuzzy --multi=$multi $prompt
    } else {
        $items | input list --fuzzy --multi=$multi --display $display $prompt
    }
}

# Record a directory visit. Each statement is atomic, so concurrent shells
# never lose updates; at worst their decay and bump steps interleave.
export def add-dir [dir: string] {
    let db = (open-dirs-db)
    $db | query db "UPDATE dirs SET score = score * ?" -p [$dir_score_decay]
    $db | query db "INSERT INTO dirs (path, score) VALUES (?, ?) ON CONFLICT (path) DO UPDATE SET score = score + excluded.score" -p [$dir $dir_score_increment]
}

# Pick a command from history and insert it into the command line.
export def cmd [] {
    let picked = (recent-commands | pick HISTORY yellow)
    if ($picked | is-empty) { return }
    if (commandline | is-empty) {
        commandline edit --insert $picked
    } else {
        commandline edit --insert $"\n($picked)"
    }
}

# Pick a directory from history and change to it.
export def --env dir [] {
    let here = $env.PWD
    let picked = (
        scored-dirs
        | get path
        | where {|d| $d != $here and ($d | path type) == dir }
        | pick LOCATION green --display {|d| $d | tilde-abbr }
    )
    if ($picked | is-not-empty) { cd $picked }
}

# Pick commands to delete from history.
export def forget-cmd [] {
    let picked = (recent-commands | pick "FORGET HISTORY" yellow --multi)
    if ($picked | is-empty) { return }
    let db = (open $nu.history-path)
    for c in $picked {
        $db | query db "DELETE FROM history WHERE command_line = ?" -p [$c]
    }
    print $"forgot ($picked | length) commands"
}

# Pick directories to delete from history.
export def forget-dir [] {
    let picked = (
        scored-dirs
        | get path
        | pick "FORGET LOCATION" green --multi --display {|d| $d | tilde-abbr }
    )
    if ($picked | is-empty) { return }
    let db = (open-dirs-db)
    for d in $picked {
        $db | query db "DELETE FROM dirs WHERE path = ?" -p [$d]
    }
    print $"forgot ($picked | length) directories"
}

# Delete directories that no longer exist from history.
export def prune-dirs [] {
    let db = (open-dirs-db)
    let invalid = ($db | query db "SELECT path FROM dirs" | get path | where {|d| ($d | path type) != dir })
    for d in $invalid {
        print $"removing invalid directory: ($d)"
        $db | query db "DELETE FROM dirs WHERE path = ?" -p [$d]
    }
    print $"removed ($invalid | length) invalid directory history entries"
}

# One-off migration of Elvish's command and directory history.
#
# Commands are imported once each, at their latest position, which is all the
# picker shows. Rerunning keeps the higher score for directories but appends
# commands again.
export def import-elvish [] {
    let tmp = (mktemp -d)
    let script = ($tmp | path join dump.elv)
    let out = ($tmp | path join out.json)
    # The store: module is interactive-only, so run the dump as Elvish's RC file.
    [
        "use store"
        "var cmds = [(store:cmds 0 -1 | each {|c| put $c[text] })]"
        "var dirs = [(store:dirs | each {|d| put [&path=$d[path] &score=$d[score]] })]"
        "put [&cmds=$cmds &dirs=$dirs] | to-json > $E:RECALL_ELVISH_EXPORT"
        "exit"
    ] | str join "\n" | save $script
    with-env { RECALL_ELVISH_EXPORT: $out } { ^elvish -rc $script }
    let export = (open $out)
    rm -r $tmp

    let cmds = ($export.cmds | reverse | uniq | reverse)
    $cmds | history import

    let db = (open-dirs-db)
    let dirs = ($export.dirs | where {|d| $d.path | str starts-with "/" })
    for d in $dirs {
        $db | query db "INSERT INTO dirs (path, score) VALUES (?, ?) ON CONFLICT (path) DO UPDATE SET score = max(score, excluded.score)" -p [$d.path $d.score]
    }
    print $"imported ($cmds | length) commands and ($dirs | length) directories"
}

export-env {
    $env.config.hooks.env_change.PWD = (
        $env.config.hooks.env_change.PWD? | default [] | append {|before, after|
            # Startup sets PWD too; only real directory changes count as visits.
            if $before != null {
                try { add-dir $after }
            }
        }
    )
}
