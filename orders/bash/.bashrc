# shellcheck shell=bash

eval "$(~/.local/bin/van/shellenv)"

# Keep short paths whole; otherwise retain the last two directories.
vanilla_prompt() {
  local directory=${1:-$PWD}
  if [[ $directory == "$HOME" ]]; then
    directory='~'
  elif [[ $directory == "$HOME/"* ]]; then
    directory=\~/"${directory#"$HOME/"}"
  fi

  if (( ${#directory} > 40 )) && [[ ${directory#*/} == */*/* ]]; then
    local last=${directory##*/}
    local parent=${directory%/*}
    parent=${parent##*/}
    directory="${directory%%/*}/.../$parent/$last"
  fi
  VANILLA_PROMPT_DIRECTORY_PREFIX=$directory
  VANILLA_PROMPT_DIRECTORY_LAST=''
  if [[ $directory == */* && -n ${directory%/*} ]]; then
    VANILLA_PROMPT_DIRECTORY_PREFIX="${directory%/*}/"
    VANILLA_PROMPT_DIRECTORY_LAST=${directory##*/}
  fi
}

PROMPT_COMMAND=vanilla_prompt
if [[ -n ${SSH_CONNECTION:-}${SSH_CLIENT:-}${SSH_TTY:-} ]]; then
  PS1='[ssh \h] \[\e[0;94m\]${VANILLA_PROMPT_DIRECTORY_PREFIX}\[\e[1m\]${VANILLA_PROMPT_DIRECTORY_LAST}\[\e[0m\]\n\[\e[1;33m\]bsh❯\[\e[0m\] '
else
  PS1='\[\e[0;94m\]${VANILLA_PROMPT_DIRECTORY_PREFIX}\[\e[1m\]${VANILLA_PROMPT_DIRECTORY_LAST}\[\e[0m\]\n\[\e[1;33m\]bsh❯\[\e[0m\] '
fi

case $- in
  *i*)
    g() {
      command alias-g "$@"
    }
    ;;
esac
