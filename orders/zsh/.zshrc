# shellcheck shell=zsh
# interactive shell
if [[ -n ${SSH_CONNECTION:-}${SSH_CLIENT:-}${SSH_TTY:-} ]]; then
  PROMPT=$'[ssh %m] %b%k%F{12}%1v%B%2v%b%f\n%B%F{red}zsh❯%f%b '
else
  PROMPT=$'%b%k%F{12}%1v%B%2v%b%f\n%B%F{red}zsh❯%f%b '
fi

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
  # %v inserts each component literally, without treating directory text as prompt code.
  psvar[1]=$directory
  psvar[2]=''
  if [[ $directory == */* && -n ${directory%/*} ]]; then
    psvar[1]="${directory%/*}/"
    psvar[2]=${directory##*/}
  fi
}

autoload -Uz add-zsh-hook
add-zsh-hook precmd vanilla_prompt
vanilla_prompt
RPROMPT=''

if [[ -o interactive ]]; then
  g() {
    command alias-g "$@"
  }
fi
