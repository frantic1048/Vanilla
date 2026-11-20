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

# Screen lines of one picker row. A row with `lines` (lists of {text, style}
# segments) renders those, one screen line each; otherwise its `display`. The
# focused row is drawn in the picker theme's row colors, matches bold;
# elsewhere matches get a chip and restore their segment's style afterwards.
def render-row [row: record, focused: bool, spec: record, words: list, columns: int] {
    let row_lines = if ($row.lines? | is-not-empty) {
        $row.lines
    } else {
        [[{text: $row.display, style: null}]]
    }
    if $focused {
        let style = (ansi {fg: $spec.theme.row_fg, bg: $spec.theme.row_bg})
        let on = (ansi {fg: $spec.theme.match_fg, bg: $spec.theme.row_bg, attr: b})
        let pad = ("" | fill -w $columns)
        $row_lines | each {|segs|
            let text = ($segs | get text | str join | highlight $words $on $"(ansi reset)($style)")
            $"($style) ($text)($pad)(ansi reset)"
        }
    } else {
        $row_lines | each {|segs|
            let parts = ($segs | each {|seg|
                let style = if $seg.style == null { "" } else { ansi $seg.style }
                $"($style)($seg.text | highlight $words (ansi $match_style) $'(ansi reset)($style)')(ansi reset)"
            })
            $" ($parts | str join)"
        }
    }
}

def row-height [row: record] {
    $row.lines? | default [[]] | length
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
        # Tippy stays regular weight: Pyonpyon has no bold face to fall back to.
        $"(ansi {fg: $spec.theme.title_fg, bg: $spec.theme.title_bg}) (tippy-mark)(ansi {fg: $spec.theme.title_fg, bg: $spec.theme.title_bg, attr: b})($spec.title) (ansi reset)"
        $"(ansi {fg: $spec.theme.count_fg, bg: $spec.theme.count_bg}) ($shown_text) of ($total) (ansi reset)"
    ]
    let left = $"($badge | str join) ($query)(ansi {attr: r}) (ansi reset)"
    let hints = (
        $spec.hints
        | each {|h| $"(ansi {attr: r}) ($h.key) (ansi reset) ($h.label)" }
        | str join "  "
    )
    let fits = ($left | visible-width) + ($hints | visible-width) + 2 <= $size.columns
    let header = if $fits and ($hints | is-not-empty) {
        # Right-align; the line was cleared when it started.
        $"($left)\e[1;($size.columns - ($hints | visible-width) + 1)H($hints)"
    } else {
        $left
    }
    let budget = $size.rows - 1
    mut body = []
    mut i = $top
    while $i < $shown and ($body | length) < $budget {
        $body = ($body | append (render-row ($rows | get $i) ($i == $sel) $spec $words $size.columns))
        $i += 1
    }
    let body = ($body | first $budget)
    # Clear each line before drawing it. Clearing after it would erase a
    # character written to the last column: with autowrap off the cursor stays
    # on that column, and erase-to-end includes the cursor position.
    let lines = ([$header] | append $body | each {|l| $"\e[2K($l)" })
    $"\e[?2026h\e[H($lines | str join "\r\n")\e[J\e[?2026l"
}

# The picker loop. `source` maps a query to rows of {value, display}, with
# optional `lines` (styled segments, see render-row) to draw instead;
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
        while $top < $sel and ($rows | skip $top | first ($sel - $top + 1) | each {|r| row-height $r } | math sum) > $height {
            $top += 1
        }
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

# Tippy from the Pyonpyon font (orders/pyonpyon), followed by two spaces: the
# glyph is square and terminals such as WezTerm and kitty let it grow into the
# first one. Shown everywhere except the Linux console, which has no font
# fallback; PYONPYON=1 or 0 overrides that.
def tippy-mark [] {
    let enabled = ($env.PYONPYON? | default (if $env.TERM? == "linux" { "0" } else { "1" }))
    if $enabled == "1" { "\u{100000}  " } else { "" }
}

# Picker themes: the title and count badges, and the focused row with its
# matches. LOCATION follows Chino (aqua hair tips into a royal navy hat),
# HISTORY follows Syaro (golden hair over dark brown).
const location_theme = {
    title_fg: "#0f1d4f", title_bg: "#62d0f0"
    count_fg: "#bfeeff", count_bg: "#1e2f7a"
    row_fg: "#f2f8ff", row_bg: "#2e4fb8", match_fg: "#8fe6ff"
}
const history_theme = {
    title_fg: "#2a1a08", title_bg: "#ecc054"
    count_fg: "#f3d27a", count_bg: "#3b2a16"
    row_fg: "#fff6dc", row_bg: "#9c6f1c", match_fg: "#ffe17a"
}

# Run the picker on the alternate screen. Returns the picked value, or null
# when cancelled. `--limit` is the most rows `source` returns, with `--count`
# giving the real total.
def picker [
    title: string
    theme: record
    source: closure
    --forget: closure
    --hints: list = []
    --limit: int
    --count: closure
] {
    let spec = {title: $title, theme: $theme, hints: $hints, limit: $limit}
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

# Gutter marks group each row. Box-drawing characters are drawn by the terminal
# to fill their cell exactly, so they align across lines and fonts, unlike Nerd
# Font icons, which WezTerm lets overflow into a following space. A worktree's
# two lines form a bracket (heavy vertical, light horizontal); each history
# directory gets a one-line bracket of its own. That one is a font glyph rather
# than a box-drawing character, so its shape depends on the font supplying it.
const bracket_open = "\u{250e} "   # ┎ down heavy and right light
const bracket_close = "\u{2516} "  # ┖ up heavy and right light
const folder_mark = "\u{2997} "    # ⦗ left black tortoise shell bracket
# Secondary parts share GitHub dark's muted foreground: quieter than the text,
# still visible on the dark background (unlike bright black or a thin dim glyph).
const muted_style = {fg: "#7d8590"}
const worktree_mark_style = {fg: yellow}
const folder_mark_style = $muted_style
const container_style = $muted_style
const home_style = {attr: b}
const branch_style = {fg: light_blue, attr: i}

# Worktrees of the repository around the current directory as two-line picker
# rows (name, then branch, joined by a bracket in the gutter), home checkout first, then by directory history
# score. Empty outside a repository.
#
# Names are relative to the collection (the home checkout's parent): the home
# checkout shows its key, its `<key>@<task>` siblings show `@<task>`, and a
# container directory such as `<key>.noindex/` is dimmed.
def worktree-entries [] {
    let r = (^git worktree list --porcelain | complete)
    if $r.exit_code != 0 { return [] }
    let entries = (
        $r.stdout
        | split row "\n\n"
        | where {|block| $block | str starts-with "worktree " }
        | each {|block|
            let fields = ($block | lines | parse "{k} {v}")
            {
                path: ($fields | where k == worktree | get -o 0.v)
                branch: ($fields | where k == branch | get -o 0.v | default "(detached)" | str replace "refs/heads/" "")
                bare: ($block | lines | any {|l| $l == bare })
            }
        }
        | where {|w| not $w.bare and ($w.path | path type) == dir }
    )
    if ($entries | is-empty) { return [] }
    let home = ($entries | first | get path)
    let collection = ($home | path dirname)
    let own = $"($home | path basename)@"
    let scores = (scored-dirs | where path in ($entries | get path))
    let named = ($entries | enumerate | each {|it|
        let w = $it.item
        let rel = if ($w.path | str starts-with $"($collection)/") {
            $w.path | str substring (($collection | str length) + 1)..
        } else {
            $w.path | tilde-abbr
        }
        let name = if ($rel | str starts-with $own) { $rel | str substring (($own | str length) - 1).. } else { $rel }
        # Dim everything up to the part that names this worktree: the branch
        # when the path ends with it, else the last component.
        let tail = if ($name | str ends-with $w.branch) and ($name | str length) > ($w.branch | str length) {
            $w.branch
        } else {
            $name | path basename
        }
        {
            path: $w.path
            home: ($it.index == 0)
            prefix: ($name | str substring 0..<(($name | str length) - ($tail | str length)))
            tail: $tail
            branch: $w.branch
            rank: (if $it.index == 0 { 1e9 } else { $scores | where path == $w.path | get -o 0.score | default 0 })
        }
    })
    $named | sort-by -r rank | each {|n|
        let name_style = if $n.home { $home_style } else { null }
        let first = [{text: $bracket_open, style: $worktree_mark_style}, {text: $n.prefix, style: $container_style}, {text: $n.tail, style: $name_style}]
            | where {|seg| $seg.text != "" }
        let second = [{text: $bracket_close, style: $worktree_mark_style}, {text: $n.branch, style: $branch_style}]
        {path: $n.path, lines: [$first $second], display: $"($n.prefix)($n.tail) ($n.branch)"}
    }
}

# Rows for the location picker: the current repository's worktrees (first,
# two lines each) and then existing history directories other than the current one,
# highest score first. Worktree paths are not repeated among history rows.
def dir-rows [query: string, worktrees: list] {
    let words = (filter-words $query)
    let here = $env.PWD
    let tree_rows = (
        $worktrees
        | where path != $here
        | where {|w| $w.display | matches-all $words }
        | each {|w| {value: $w.path, display: $w.display, lines: $w.lines} }
    )
    let tree_paths = ($worktrees | get -o path | default [])
    let history = (
        scored-dirs
        | where path != $here
        | where path not-in $tree_paths
        | insert display {|d| $d.path | tilde-abbr }
        | where {|d| $d.display | matches-all $words }
        | where {|d| ($d.path | path type) == dir }
    )
    let history_rows = ($history | each {|d| {
        value: $d.path
        display: $d.display
        lines: [[{text: $folder_mark, style: $folder_mark_style}, {text: $d.display, style: null}]]
    } })
    $tree_rows | append $history_rows
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
        picker HISTORY $history_theme {|q| cmd-rows $q }
            --forget {|c| forget-cmd-entry $c }
            --limit $cmd_picker_limit
            --count { count-commands }
            --hints [
                {key: Enter, label: "insert"}
                {key: Ctrl-D, label: "forget"}
            ]
    )
    if $picked == null { return }
    if (commandline | is-empty) {
        commandline edit --insert $picked
    } else {
        commandline edit --insert $"\n($picked)"
    }
}

# Pick a directory from history and change to it. Inside a git repository its
# worktrees come first. Ctrl-D forgets the highlighted directory.
export def --env dir [] {
    let worktrees = (worktree-entries)
    let picked = (
        picker LOCATION $location_theme {|q| dir-rows $q $worktrees }
            --forget {|d| forget-dir-entry $d }
            --hints [
                {key: Enter, label: "go to"}
                {key: Ctrl-D, label: "forget"}
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
