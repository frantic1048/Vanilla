#!/usr/bin/env nu

def with-external-commands-do [commands: list<string>, action: closure]: nothing -> nothing {
    let missing_commands = $commands | where {|cmd| which $cmd | is-empty }
    if ($missing_commands | is-empty) {
        do $action
    }
}

def print-section-title [title: string]: nothing -> nothing {
    print $"(ansi green_bold)|> ($title)(ansi reset)"
}

def print-and-do-command [cmd: list<string>]: nothing -> nothing {
    print $"(ansi default_bold)->(ansi reset) ($cmd | str join " ")"
    ^$cmd
}

# Download some disk space for current machine *_*
# No and never do heuristics, just dumb cleanup of known caches and temporary files.
def main [
  --execute # Actually execute the cleanup
  --urgent # More aggressive approach
] {
    if not $execute {
        print "noop: pass --help to see usage"
        return
    }

    print-section-title "pnpm"
    if not $urgent {
        with-external-commands-do ["pnpm"] {||
            print-and-do-command [pnpm store prune --force]
        }
    } else {
        print-and-do-command [
            rm
            -rf
            ("~/.local/share/pnpm/store" | path expand)
        ]
        print-and-do-command [
            rm
            -rf
            ("~/Library/pnpm/store" | path expand)
        ]
    }

    with-external-commands-do ["deno"] {||
        print-section-title "deno"
        print-and-do-command [deno clean]
    }

    with-external-commands-do ["npm"] {||
        print-section-title "npm"
        if not $urgent {
            print "noop when --urgent is not set."
        } else {
            print-and-do-command [npm cache clean --force]
        }
    }

    with-external-commands-do ["yarn"] {||
        print-section-title "yarn"
        ^yarn cache clean --mirror
    }

    with-external-commands-do ["docker"] {||
        print-section-title "docker"
        if not $urgent {
            print-and-do-command [docker system prune --force]
        } else {
            print-and-do-command [docker system prune --all --force]
        }
    }

    with-external-commands-do ["brew"] {||
        print-section-title "brew"
        if not $urgent {
            print-and-do-command [brew cleanup]
        } else {
            print-and-do-command [brew cleanup --prune=all]
        }
    }
}
