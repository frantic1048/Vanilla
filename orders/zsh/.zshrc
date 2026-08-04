# shellcheck shell=zsh
# interactive shell
eval "$(starship init zsh)"

if [[ -o interactive ]]; then
  g() {
    command alias-g "$@"
  }
fi
