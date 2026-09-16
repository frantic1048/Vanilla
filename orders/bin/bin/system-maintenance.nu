#!/usr/bin/env nu

const script_path = path self .

# MEMO: call `path expand` to resolve symlinks first
let work_dir = (
    $script_path
    | path expand
    | path dirname
    | path dirname
    | path dirname
)

def env_or [name: string, fallback: string]: nothing -> string {
    if $name in ($env | columns) {
        $env | get $name
    } else {
        $fallback
    }
}

def print_heading [title: string] {
    print $"== ($title) =="
}

def on_os_do [oses: list<string>, block: closure] {
    if $nu.os-info.name in $oses {
        do $block
    }
}

def in_repo [block: closure] {
    cd $work_dir
    do $block
}

def "main homebrew" [] {
    in_repo {
        on_os_do [macos] {
            print_heading "Homebrew"
            brew update
            brew bundle install --upgrade
        }
    }
}

def "main pacman" [] {
    in_repo {
        on_os_do [linux] {
            print_heading "Pacman"
            paru -Sy archlinux-keyring archlinuxcn-keyring
            paru -Syuw
            paru -Syu

            let result = paru -Qdtq | complete
            if $result.exit_code == 0 {
                let orphans = $result.stdout | lines
                if ($orphans | is-not-empty) {
                    paru -Rns ...$orphans
                }
            }
        }
    }
}

def "main proto" [] {
    print_heading "Proto"
    ^proto clean --yes --days 60
    ^proto upgrade --yes

    # We only bump global tools,
    # thus we run these command outside of the repo
    cd $nu.home-dir
    ^proto outdated --config-mode upwards-global --yes --update
    ^proto install --config-mode upwards-global --pin global --yes
}

def "main claude" [] {
    if (which claude | is-not-empty) {
        print_heading "Claude"
        ^claude update
    }
}

def "main blend" [] {
    in_repo {
        print_heading "Dotfiles"

        let blend_exe = (env_or BLEND_EXE "blend")
        let blend_dir = (env_or BLEND_DIR $work_dir)
        let blend_home = (env_or BLEND_HOME $nu.home-dir)

        ^$blend_exe "--blend-dir" $blend_dir "--home" $blend_home sync
    }
}

def "main cargo" [] {
    in_repo {
        print_heading "Cargo"
        ^cargo install nickel-lang-lsp
        ^cargo install --git https://github.com/nushell/nufmt
    }
}

def main [] {
    print_heading "Starting system maintenance"

    main homebrew
    main pacman
    main proto
    main claude
    main blend
    main cargo

    print_heading "System maintenance completed"
}
