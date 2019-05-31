# env vars
paths = [
  {~}/bin
  {~}/npm-global/bin
  {~}/.gem/ruby/2.2.0/bin
  {~root}/.composer/vendor/bin
  $@paths
]

set-env NODE_PATH (joins : [
  {~}/npm-global/lib/node_modules
  /usr/lib/node_modules
  (splits : $E:NODE_PATH)
])

set-env VISUAL nano

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
