# shellcheck shell=dash

eval "$(~/.local/bin/van/shellenv)"
eval "$(starship init bash)"

case $- in
  *i*)
    g() {
      command alias-g "$@"
    }
    ;;
esac
