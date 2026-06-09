use str
use path
use kokkoro

var existing-dir~ = $kokkoro:existing-dir~
var at-env~ = $kokkoro:at-env~

fn ignore-error {|callback~|
  try {
    callback
  } catch e {
  }
}

fn succeeds {|callback~|
  try {
    callback
    put $true
  } catch e {
    put $false
  }
}

# PATH
at-env &os="darwin" {
  # MacOS

  # https://nixos.org/manual/nix/stable/installation/env-variables.html
  var nixPaths = [
    {~}/.nix-profile/bin
    /nix/var/nix/profiles/default/bin
  ]

  set paths = [
    ~/.local/bin/van

    (existing-dir ~/.local/share/mise/shims)

    (existing-dir ~/.go/bin)

    # CLI form Rancher Desktop
    (existing-dir ~/.rd/bin)

    # npm prefix
    (existing-dir ~/.npm-global/bin) # legacy
    (existing-dir ~/.local/share/npm/bin) # current

    $@nixPaths

    # GnuPG for OS X
    # https://sourceforge.net/p/gpgosx/docu/Download/
    (existing-dir /usr/local/gnupg-2.4/bin)

    # tk in MacOS is broken :(
    #
    # check `brew info tcl-tk`
    # https://superuser.com/questions/1696372/wish-based-tools-git-gui-gitk-showing-broken-black-ui-on-macos-monterey
    (existing-dir /usr/local/opt/tcl-tk/bin)

    # general local bin
    (existing-dir ~/.local/bin)

    # homebrew(x86_64), and other binaries
    (existing-dir /usr/local/bin)
    # homebrew(apple silicon)
    (existing-dir /opt/homebrew/bin)

    $@paths
  ]
}

# Go
at-env &os="darwin" {
  set-env GOPATH {~}/.go
  set-env GOBIN {~}/.go/bin
}

# Nix
at-env &os="darwin" {
  set-env NIX_SSL_CERT_FILE /etc/ssl/certs/ca-certificates.crt
  set-env NIX_PROFILES (str:join " " [
    /nix/var/nix/profiles/default
    /Users/(whoami)/.nix-profile
  ])
}

at-env &os="linux" {
  # arch linux
  set paths = [
    {~}/.local/bin/van
    {~}/.local/bin
    {~}/.nix-profile/bin
    {~}/.go/bin
    {~}/.gem/ruby/2.2.0/bin
    /usr/bin/vendor_perl
    $@paths
  ]
}

# Proto
# https://moonrepo.dev/proto
set-env PROTO_HOME {~}/.proto
set paths = [
  $E:PROTO_HOME/shims
  $E:PROTO_HOME/bin
  $E:PROTO_HOME/tools/node/globals/bin
  $@paths
]

# pnpm
set-env PNPM_HOME {~}/.local/share/pnpm
set paths = [$E:PNPM_HOME $@paths]

# Node.js
set-env NODE_OPTIONS $E:NODE_OPTIONS' --max-old-space-size=8192'

# Rust
set-env RUSTUP_HOME {~}/.rustup
set-env CARGO_HOME {~}/.cargo
set paths = [{~}/.cargo/bin $@paths]

if (has-external nvim) {
  set-env VISUAL nvim
} else {
  set-env VISUAL nano
}

# Codex
# Making `codex exec` work with custom endpoints
set-env CODEX_INTERNAL_ORIGINATOR_OVERRIDE 'codex_cli_rs'

# GPG
if (has-external gpgconf) {
  at-env &os="darwin" {
    if (not (succeeds { pgrep gpg-agent >/dev/null 2>&1 })) {
        ignore-error { gpgconf --launch gpg-agent >/dev/null 2>&1 }
    }

    # macOS sometimes set SSH_AUTH_SOCK to a weird path like
    # /private/tmp/com.apple.launchd.abcdefg123/Listeners
    # which is listened by ssh-agent, but not gpg-agent
    # (check with `lsof /private/tmp/com.apple.launchd.abcdefg123/Listeners`)
    # so we need to set it to the correct path every time :(
    if (and (succeeds { pgrep gpg-agent >/dev/null 2>&1 }) (succeeds { gpgconf --list-dirs agent-ssh-socket >/dev/null 2>&1 })) {
        set-env SSH_AUTH_SOCK (gpgconf --list-dirs agent-ssh-socket)
    }

    if (and (not (has-env GPG_TTY)) (succeeds { tty >/dev/null 2>&1 })) {
        set-env GPG_TTY (tty)
    }
  }

  at-env &os="linux" {
    if (not (succeeds { pidof gpg-agent >/dev/null 2>&1 })) {
        ignore-error { gpgconf --launch gpg-agent >/dev/null 2>&1 }
    }

    # FIXME:
    # GNOME Keyring's gcr-ssh-agent is setting this to an unusable value......
    # https://bbs.archlinux.org/viewtopic.php?id=293602
    # systemctl --user mask gcr-ssh-agent.socket
    if (succeeds { gpgconf --list-dirs agent-ssh-socket >/dev/null 2>&1 }) {
        set-env SSH_AUTH_SOCK (gpgconf --list-dirs agent-ssh-socket)
    }

    if (and (not (has-env GPG_TTY)) (succeeds { tty >/dev/null 2>&1 })) {
      # Configure pinentry to use the correct TTY
      # https://wiki.archlinux.org/index.php/GnuPG#Configure_pinentry_to_use_the_correct_TTY
      set-env GPG_TTY (tty)
      ignore-error { gpg-connect-agent updatestartuptty /bye >/dev/null 2>&1 }
  }
}
}

# launchd is weird
at-env &os="darwin" {
  # this is dumb, but...
  # https://stackoverflow.com/questions/135688/setting-environment-variables-on-os-x
  each {|env_name|
    if (and (has-env $env_name) (has-external launchctl)) {
      var current_value = ''
      ignore-error { set current_value = (launchctl getenv $env_name 2>/dev/null) }

      if (not (==s $current_value (get-env $env_name))) {
        ignore-error { launchctl setenv $env_name (get-env $env_name) >/dev/null 2>&1 }
      }
    }
  } ['PATH' 'PROTO_HOME' 'SHELL']
}

# https://docs.grit.io/cli/quickstart#telemetry
set-env GRIT_TELEMETRY_DISABLED true

# Load local envs
if (src)[is-file] {
  var local_env_file = (path:join (path:dir (src)[name]) 'env.local.elv')
  if (not (path:is-regular $local_env_file)) {
     cp $local_env_file'.example' $local_env_file
  }
  use ./env.local
}