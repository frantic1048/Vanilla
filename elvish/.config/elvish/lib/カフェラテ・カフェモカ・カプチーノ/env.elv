use str

use kokkoro

var existing-dir~ = $kokkoro:existing-dir~
var at-env~ = $kokkoro:at-env~


# PATH
at-env &os="darwin" {
  # MacOS

  # https://nixos.org/manual/nix/stable/installation/env-variables.html
  var nixPaths = [
    {~}/.nix-profile/bin
    /nix/var/nix/profiles/default/bin
    /run/current-system/sw/bin
  ]

  set paths = [
    {~}/bin
    {~}/.n/bin
    {~}/.npm-global/bin

    {~}/go/bin
    $@nixPaths

    # kitty shell
    (existing-dir /Applications/kitty.app/Contents/MacOS)

    # GnuPG for OS X
    # https://sourceforge.net/p/gpgosx/docu/Download/
    (existing-dir /usr/local/gnupg-2.4/bin)

    # tk in MacOS is broken :(
    #
    # check `brew info tcl-tk`
    # https://superuser.com/questions/1696372/wish-based-tools-git-gui-gitk-showing-broken-black-ui-on-macos-monterey
    (existing-dir /usr/local/opt/tcl-tk/bin)

    # homebrew, mainly
    /usr/local/bin
    $@paths
  ]
}

# Go
at-env &os="darwin" {
  set-env GOPATH {~}/go
  set-env GOBIN {~}/go/bin
}

# Nix
at-env &os="darwin" {
  set-env NIX_PATH (str:join : [
    darwin-config={~}/.nixpkgs/darwin-configuration.nix
    {~}/.nix-defexpr/channels
    # (if (has-env NIX_PATH) { put $E:NIX_PATH })
  ])
  #set-env NIX_SSL_CERT_FILE /etc/ssl/certs/ca-certificates.crt
  set-env NIX_PROFILE_DIR /nix/var/nix/profiles/per-user/(whoami)
  set-env NIX_PROFILES (str:join " " [
    /nix/var/nix/profiles/default
    /run/current-system/sw
    /Users/(whoami)/.nix-profile
  ])
  set-env NIX_REMOTE daemon
}

at-env &os="linux" {
  # arch linux
  set paths = [
    {~}/bin
    {~}/.local/bin
    {~}/npm-global/bin
    {~}/go/bin
    {~}/.gem/ruby/2.2.0/bin
    /usr/bin/vendor_perl
    $@paths
  ]
}

# Node
at-env &os="darwin" {
  set-env N_PREFIX {~}/.n
  set-env PNPM_HOME {~}/Library/pnpm

  set paths = [
    $E:PNPM_HOME
    $@paths
  ]
}

set-env VISUAL nano

# GPG
at-env &os="darwin" {
  if (not ?(pgrep gpg-agent >&-)) {
      gpgconf --launch gpg-agent > /dev/null
  }

  # macOS sometimes set SSH_AUTH_SOCK to a weird path like
  # /private/tmp/com.apple.launchd.abcdefg123/Listeners
  # which is listened by ssh-agent, but not gpg-agent
  # (check with `lsof /private/tmp/com.apple.launchd.abcdefg123/Listeners`)
  # so we need to set it to the correct path every time :(
  if ?(pgrep gpg-agent >&-) {
      set-env SSH_AUTH_SOCK (gpgconf --list-dirs agent-ssh-socket)
  }

  if (not (has-env GPG_TTY)) {
      set-env GPG_TTY (tty)
  }
}

at-env &os="linux" {
  if (not ?(pidof gpg-agent)) {
      gpgconf --launch gpg-agent > /dev/null
  }

  if (not (has-env SSH_AUTH_SOCK)) {
      set-env SSH_AUTH_SOCK (gpgconf --list-dirs agent-ssh-socket)
  }

  if (not (has-env GPG_TTY)) {
      # Configure pinentry to use the correct TTY
      # https://wiki.archlinux.org/index.php/GnuPG#Configure_pinentry_to_use_the_correct_TTY
      set-env GPG_TTY (tty)
      gpg-connect-agent updatestartuptty /bye >/dev/null
  }
}

# launchd is weird
at-env &os="darwin" {
  # this is dumb, but...
  # https://stackoverflow.com/questions/135688/setting-environment-variables-on-os-x
  each {|env_name|
  if (not (and ^
      (== (count [(launchctl getenv $env_name)]) 1) ^
      (==s (launchctl getenv $env_name) (get-env $env_name)) ^
      )) {
        launchctl setenv $env_name (get-env $env_name)
      }
  } ['PATH' 'PNPM_HOME' 'N_PREFIX' 'SHELL']
}
