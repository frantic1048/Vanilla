use str
use kokkoro

var at-env~ = $kokkoro:at-env~

# PATH
at-env &os="darwin" {
  # MacOS
  set paths = [
    {~}/bin
    {~}/.n/bin
    {~}/.npm-global/bin
    /usr/local/bin
    $@paths
  ]
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
  if (not ?(pgrep gpg-agent)) {
      gpgconf --launch gpg-agent > /dev/null
  }
  if (not (has-env SSH_AUTH_SOCK)) {
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