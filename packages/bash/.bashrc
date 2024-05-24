#
# ~/.bashrc
#

# Start the gpg-agent if not already running
if ! pgrep -x -u "${USER}" gpg-agent >/dev/null 2>&1; then
  gpg-connect-agent /bye >/dev/null 2>&1
fi

# Set SSH to use gpg-agent
unset SSH_AGENT_PID
if [ "${gnupg_SSH_AUTH_SOCK_by:-0}" -ne $$ ]; then
  export SSH_AUTH_SOCK="/run/user/$UID/gnupg/S.gpg-agent.ssh"
fi

# Set GPG TTY
export GPG_TTY=$(tty)

# Refresh gpg-agent tty in case user switches into an X session
gpg-connect-agent updatestartuptty /bye >/dev/null


## PS1 setting
#PS1='⌊\u@\W⌋\$'
function _update_ps1() {
    PS1="$(~/bin/powerline-shell.py $? 2> /dev/null)"
}

if [ "$TERM" != "linux" ]; then
    PROMPT_COMMAND="_update_ps1; $PROMPT_COMMAND"
fi

# If not running interactively, don't do anything
[[ $- != *i* ]] && return

source /etc/profile

setZHLocale ()
{
  LANG=zh_CN.UTF-8
}

gitDebugOn ()
{
  # for  git truobleshooting
  export GIT_TRACE_PACKET=1
  export GIT_TRACE=1
  export GIT_CURL_VERBOSE=1
}

gitDebugOff ()
{
  export GIT_TRACE_PACKET=0
  export GIT_TRACE=0
  export GIT_CURL_VERBOSE=0
}

node_prof_process () {
  for log in $(ls isolate*.log); do
    node --prof-process $log > ${log%.log}.txt
  done
}

#count files of folder
file_count () {
    find "$@" -type f | wc -l
}

#default editor
export VISUAL="nano"

if [ $XDG_CURRENT_DESKTOP = "i3" ]; then
    export QT_QPA_PLATFORMTHEME="kde"
fi

#node.js module path
export NODE_PATH=~/npm-global/lib/node_modules/:$NODE_PATH:/usr/lib/node_modules

export PATH=$PATH:~/npm-global/bin/:~/.gem/ruby/2.2.0/bin/:/root/.composer/vendor/bin:~/bin/
alias fakePATH="export PATH=~/bin/:$PATH"

# export GIT_ASKPASS="/usr/bin/ksshaskpass"
# export SSH_ASKPASS="/usr/bin/ksshaskpass"

# env for armitage
export MSF_DATABASE_CONFIG="~/.msf4/database.yml"
# start msfrpcd for armitage
export msfrpcdes="msfrpcd -f -U msf -P msf -S -p 55559"

# FIX phantomjs crash issue
# https://github.com/ariya/phantomjs/issues/14061
alias phantomjs="QT_QPA_PLATFORM='' phantomjs"

# task name auto completion for gulp
# toooo slow, disabled
#eval "$(gulp --completion=bash)"

# init nvm
source /usr/share/nvm/init-nvm.sh

alias ls='ls --color=auto'
alias tts='text-to-speech zh-CN'
alias ttse='text-to-speech'
alias ttsj='text-to-speech ja'
alias prpr='proxychains'
alias prprme='prpr bash'

# simpler python http server
alias pyserv='python -m http.server'

# start hefur bittorrent tracker
alias tracker='hefurd -ipv6 -log-color -log-level info -udp-port 6969 -http-port 6969 -https-port 6970'

alias curl-prpr="alias curl=\"curl -x localhost:8388\""
alias curl-unprpr="unalias curl"

# update package.json's version without git command
alias nv1="npm --froce --no-git-tag-version version major && git add --all"
alias nv2="npm --froce --no-git-tag-version version minor && git add --all"
alias nv3="npm --froce --no-git-tag-version version patch && git add --all"

# aria2c alias
alias aria="aria2c --conf-path=/home/chino/bkped/aria2c.conf "

# extract non-utf8 archive, like gbk..
alias xagbk="env LANG=C 7z x "
alias mvgbk="convmv -f gbk -t utf8 "

# btsync control
alias btsyncStart="systemctl --user start btsync"
alias btsyncStop="systemctl --user stop btsync"
alias btsyncRestart="systemctl --user restart btsync"

# browser-sync
# https://browsersync.io/
alias serve="browser-sync start --server"

# package manage
alias ya=pacaur
alias pa=pacman

# git
alias such=git
alias very=git
alias gs="git status"
alias gdf="git diff HEAD"
alias gadd="git add"
alias grst="git reset"
alias gcm="git commit"
alias gcz="git cz"
alias gmrg="git merge --ff-only"
alias gr="git rebase"
alias grcon="git rebase --continue"
alias grabt="git rebase --abort"
alias gsign-on="git config commit.gpgsign true"
alias gsign-off="git config commit.gpgsign false"

# npm
alias nr="npm run"
alias ni="npm install"
alias nt="npm test"
alias nrst="rm -rf ./node_modules && npm install"

# count pdf chars
wcpdf () {
    pdftotext "$1" -enc UTF-8 - | wc -m
}

# SDDM
alias sddm-test-theme="sddm-greeter --test-mode --theme"

# enable autocd
shopt -s autocd

# flush config
alias flush-bashrc="source ~/.bashrc"

alias fffneofetch="neofetch  --size 360px --crop_mode fill --image ~/Media/pic/sACG/misc/pixiv\ 60223242_p0.jpg  --disable model"
