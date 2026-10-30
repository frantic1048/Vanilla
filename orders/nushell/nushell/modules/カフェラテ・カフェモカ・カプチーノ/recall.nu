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

def scored-dirs [] {
    open-dirs-db | query db "SELECT path, score FROM dirs ORDER BY score DESC"
}

# The pickers filter like Elvish's listing modes: every space-separated word
# must occur as a substring, ignoring case unless the word has uppercase. The
# source order (score or recency) is kept; nothing is re-ranked.
# ref: https://github.com/elves/elvish/blob/v0.21.0/pkg/edit/filter/compile.go

const cmd_picker_limit = 500
# Matches on unfocused rows: a subtle gray chip for a dark theme.
const match_style = {fg: "#e0e0e0", bg: "#3a3a3a", attr: b}

def filter-words [query: string] {
    $query | split row " " | where $it != ""
}

def ignores-case [word: string] {
    $word == ($word | str lowercase)
}

def matches-all [words: list<string>] {
    let text = $in
    $words | all {|w|
        if (ignores-case $w) { $text | str contains -i $w } else { $text | str contains $w }
    }
}

# Wrap matches in `on`; `off` must restore the surrounding row style.
def highlight [words: list<string>, on: string, off: string] {
    let text = $in
    if ($words | is-empty) { return $text }
    let pattern = (
        $words
        | each {|w|
            let escaped = ($w | str replace -ar '[\\.+*?()|\[\]{}^$#&~-]' '\$0')
            if (ignores-case $w) { $"\(?i:($escaped)\)" } else { $escaped }
        }
        | str join "|"
    )
    $text | str replace -ar $pattern $"($on)${0}($off)"
}

# Full-screen picker: drawn with ANSI on the alternate screen, keys read one
# event at a time with `input listen`. Autowrap is off so the terminal clips
# long rows, and each frame is one synchronized write to avoid flicker.
const picker_enter = "\e[?1049h\e[?25l\e[?7l"
const picker_leave = "\e[?7h\e[?25h\e[?1049l"

# Normalize a key event to a chord like `ctrl+n`, `alt+backspace` or `A`.
def key-chord [] {
    let e = $in
    let mods = (
        $e.modifiers
        | each { str replace -r '^keymodifiers\((.*)\)$' '$1' }
        | where $it != shift # already folded into the char
        | each {|m| if $m == control { "ctrl" } else { $m } }
    )
    [...$mods $e.code] | str join "+"
}

def visible-width [] {
    ansi strip | str length -g
}

# `spec` holds the fixed parts: title, color, hints and the row limit. The
# title badge counts shown/total rows, padded to a fixed width so nothing
# shifts as it changes; the shown count turns bold when the source hit its
# limit. `hints` are {key, label} records shown Elvish-style on the right,
# dropped when the query needs the room.
def render-picker [spec: record, query: string, rows: list, total: int, sel: int, top: int] {
    let size = (term size)
    let words = (filter-words $query)
    let shown = ($rows | length)
    let pad = ("" | fill -w (($total | into string | str length) - ($shown | into string | str length)))
    let shown_text = if $spec.limit != null and $shown >= $spec.limit {
        $"($pad)\e[1m($shown)\e[22m"
    } else {
        $"($pad)($shown)"
    }
    let badge = [
        $"(ansi {fg: $spec.color, attr: rb}) ($spec.title) (ansi reset)"
        $"(ansi {fg: $spec.color, bg: dark_gray}) ($shown_text) of ($total) (ansi reset)"
    ]
    let left = $"($badge | str join) ($query)(ansi {attr: r}) (ansi reset)"
    let hints = (
        $spec.hints
        | each {|h| $"(ansi {attr: r}) ($h.key) (ansi reset) ($h.label)" }
        | str join "  "
    )
    let fits = ($left | visible-width) + ($hints | visible-width) + 2 <= $size.columns
    let header = if $fits and ($hints | is-not-empty) {
        # Right-align, after clearing the rest of the line.
        $"($left)\e[K\e[1;($size.columns - ($hints | visible-width) + 1)H($hints)"
    } else {
        $left
    }
    let body = (
        $rows | skip $top | first ($size.rows - 1) | enumerate | each {|it|
            if $top + $it.index == $sel {
                # Matches use the bright variant of the row color.
                let row = (ansi {fg: $spec.color, attr: r})
                let on = (ansi {fg: $"light_($spec.color)", attr: rb})
                let display = ($it.item.display | highlight $words $on $"(ansi reset)($row)")
                let pad = ("" | fill -w $size.columns)
                $"($row) ($display)($pad)(ansi reset)"
            } else {
                $" ($it.item.display | highlight $words (ansi $match_style) (ansi reset))"
            }
        }
    )
    let lines = ([$header] | append $body | each {|l| $"($l)\e[K" })
    $"\e[?2026h\e[H($lines | str join "\r\n")\e[J\e[?2026l"
}

# The picker loop. `source` maps a query to rows of {value, display};
# `forget` deletes a row's value, bound to Ctrl-D; `count` returns the total
# behind a limited source, else the unfiltered row count is the total.
def picker-loop [spec: record, source: closure, forget: any, count: any] {
    mut query = ""
    mut rows = (do $source $query)
    mut total = if $count == null { $rows | length } else { do $count }
    mut sel = 0
    mut top = 0
    loop {
        let height = (term size).rows - 1
        let last = ([0 (($rows | length) - 1)] | math max)
        $sel = ([0 ([$sel $last] | math min)] | math max)
        if $sel < $top { $top = $sel }
        if $sel >= $top + $height { $top = $sel - $height + 1 }
        print -n (render-picker $spec $query $rows $total $sel $top)

        # Binding as `any`: `input listen` declares only the key record shape.
        let e: any = (input listen --types [key paste resize])
        let old_query = $query
        if $e.type == paste {
            $query += ($e.content | str replace -ar '\s+' " ")
        } else if $e.type == key {
            let chord = ($e | key-chord)
            match $chord {
                "enter" => {
                    if ($rows | is-not-empty) { return ($rows | get $sel | get value) }
                }
                "esc" | "ctrl+c" | "ctrl+g" => { return null }
                "up" | "ctrl+p" => { $sel -= 1 }
                "down" | "ctrl+n" => { $sel += 1 }
                "pageup" => { $sel -= $height }
                "pagedown" => { $sel += $height }
                "home" => { $sel = 0 }
                "end" => { $sel = $last }
                "backspace" => { $query = ($query | str substring -g 0..<(($query | str length -g) - 1)) }
                "ctrl+w" | "alt+backspace" => { $query = ($query | str replace -r '\S*\s*$' "") }
                "ctrl+u" => { $query = "" }
                "ctrl+d" => {
                    if $forget != null and ($rows | is-not-empty) {
                        do $forget ($rows | get $sel | get value)
                        $rows = (do $source $query)
                        $total = if $count == null { $total - 1 } else { do $count }
                    }
                }
                _ => {
                    if $e.key_type == char and ($chord | str length -g) == 1 { $query += $chord }
                }
            }
        }
        if $query != $old_query {
            $rows = (do $source $query)
            $sel = 0
            $top = 0
        }
    }
}

# Run the picker on the alternate screen. Returns the picked value, or null
# when cancelled. `--limit` is the most rows `source` returns, with `--count`
# giving the real total.
def picker [
    title: string
    color: string
    source: closure
    --forget: closure
    --hints: list = []
    --limit: int
    --count: closure
] {
    let spec = {title: $title, color: $color, hints: $hints, limit: $limit}
    print -n $picker_enter
    let outcome = try {
        {picked: (picker-loop $spec $source $forget $count)}
    } catch {|err|
        {error: $err}
    }
    print -n $picker_leave
    if "error" in $outcome { error make {msg: $outcome.error.msg} }
    $outcome.picked
}

# Rows for the location picker: existing directories other than the current
# one, highest score first.
def dir-rows [query: string] {
    let words = (filter-words $query)
    let here = $env.PWD
    scored-dirs
    | where path != $here
    | insert display {|d| $d.path | tilde-abbr }
    | where {|d| $d.display | matches-all $words }
    | where {|d| ($d.path | path type) == dir }
    | each {|d| {value: $d.path, display: $d.display} }
}

def forget-dir-entry [dir: string] {
    open-dirs-db | query db "DELETE FROM dirs WHERE path = ?" -p [$dir]
}

# Rows for the history picker: unique commands, most recent first, filtered in
# SQLite to stay fast on a large history. SQLite's lower() only folds ASCII.
# Multi-line commands show on one row with ↵ marking the line breaks.
def cmd-rows [query: string] {
    let words = (filter-words $query)
    let conditions = (
        $words | each {|w|
            if (ignores-case $w) { "instr(lower(command_line), ?) > 0" } else { "instr(command_line, ?) > 0" }
        }
    )
    let where = if ($conditions | is-empty) { "" } else { $"WHERE ($conditions | str join ' AND ')" }
    open $nu.history-path
    | query db $"SELECT command_line FROM history ($where) GROUP BY command_line ORDER BY max\(id) DESC LIMIT ($cmd_picker_limit)" -p $words
    | get command_line
    | each {|c| {value: $c, display: ($c | str replace -a "\n" " \u{21b5} ")} }
}

def count-commands [] {
    open $nu.history-path | query db "SELECT count(DISTINCT command_line) AS n FROM history" | get 0.n
}

def forget-cmd-entry [command: string] {
    open $nu.history-path | query db "DELETE FROM history WHERE command_line = ?" -p [$command]
}

# Record a directory visit. Each statement is atomic, so concurrent shells
# never lose updates; at worst their decay and bump steps interleave.
export def add-dir [dir: string] {
    let db = (open-dirs-db)
    $db | query db "UPDATE dirs SET score = score * ?" -p [$dir_score_decay]
    $db | query db "INSERT INTO dirs (path, score) VALUES (?, ?) ON CONFLICT (path) DO UPDATE SET score = score + excluded.score" -p [$dir $dir_score_increment]
}

# Pick a command from history and insert it into the command line without
# running it. Ctrl-D forgets the highlighted command.
export def cmd [] {
    let picked = (
        picker HISTORY yellow {|q| cmd-rows $q }
            --forget {|c| forget-cmd-entry $c }
            --limit $cmd_picker_limit
            --count { count-commands }
            --hints [
                {key: Enter, label: "pick focused command"}
                {key: Ctrl-D, label: "delete focused item"}
            ]
    )
    if $picked == null { return }
    if (commandline | is-empty) {
        commandline edit --insert $picked
    } else {
        commandline edit --insert $"\n($picked)"
    }
}

# Pick a directory from history and change to it. Ctrl-D forgets the
# highlighted directory.
export def --env dir [] {
    let picked = (
        picker LOCATION green {|q| dir-rows $q }
            --forget {|d| forget-dir-entry $d }
            --hints [
                {key: Enter, label: "go to focused directory"}
                {key: Ctrl-D, label: "delete focused item"}
            ]
    )
    if $picked != null { cd $picked }
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
