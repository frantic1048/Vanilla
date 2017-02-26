#
# ~/.bashrc
#

# If not running interactively, don't do anything
[[ $- != *i* ]] && return

source /etc/profile

setPS1 ()
{
  #HIKOTO=$(node -p "(`curl -ks https://api.hitokoto.us:214/rand?charset=utf-8`).hitokoto")
  local HIKOTO="少年の夢は遠い空超えて"
  PS1='⌊\u⚛\H⌋$HIKOTO\nSo, \W, as I pray\$'
}

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

#node.js module path
export NODE_PATH=/home/frantic/npm-global/lib/node_modules/:$NODE_PATH:/usr/lib/node_modules

export PATH=~/npm-global/bin/:~/.gem/ruby/2.2.0/bin/:/root/.composer/vendor/bin:~/code/using/:$PATH

export GIT_ASKPASS="/usr/bin/ksshaskpass"
export SSH_ASKPASS="/usr/bin/ksshaskpass"

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

alias ls='ls --color=auto'
alias tts='text-to-speech zh-CN'
alias ttse='text-to-speech'
alias ttsj='text-to-speech ja'
alias prpr='proxychains'

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
alias aria="aria2c --conf-path=/home/frantic/bkped/aria2c.conf "

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
alias ya=yaourt
alias pa=pacman

# git
alias such=git
alias very=git
alias gs="git status"
alias gadd="git add"
alias grs="git reset"
alias gc="git commit"
alias gcz="git cz"
alias gm="git merge --ff-only"
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

# SDDM
alias sddm-test-theme="sddm-greeter --test-mode --theme"

# waifu2x
waifu () {
    if [[ $# -eq 0 ]]; then
        cat << EOF

Usage: waifu [OPTIONS] FILE1 [FILE2 ...]

Options:
    -d          dry run, just print out the commands to exec.
    -n<1|2>     noise reduction level
    -s<NUM>     scale ratio; default: -s4
    -j<NUM>     numbers of threads launching at the same time; default: -j4
    --          terminate options list

EOF
        waifu2x --version
        waifu2x --list-processor
        return 0
    fi

    local DRYRUN=''
    local SCALE=''
    local NOISE=''
    local JOBS=''

    while [[ $# -gt 0 ]]; do
        opt="$1"

        case $opt in
            --) # terminate options list
                shift
                break
            ;;
            -d) # dry run
                DRYRUN="yes"
            ;;
            -s*) # scale ratio
                SCALE=${opt:2}
            ;;
            -n*) # noise reduction level
                NOISE=${opt:2}
            ;;
            -j*) # concurrent jobs
                JOBS=${opt:2}
            ;;
            *) # no more options
                break
            ;;
        esac
        shift
    done

    if [[ $# -eq 0 ]]; then
        echo no input file was specified, exiting.
        return 1
    fi

    local CONFIG=""
    local POSTFIX="_waifu"

    if [[ -n "$SCALE" ]]; then
        CONFIG+=" --scale_ratio $SCALE"
        POSTFIX+="_s$SCALE"
    else
        # defaults to 4x scale
        CONFIG+=" --scale_ratio 4"
        POSTFIX+="_s4"
    fi

    if [[ -n "$NOISE" ]]; then
        CONFIG+=" --noise_level $NOISE"
        POSTFIX+="_n$NOISE"
    else
        POSTFIX+="_n0"
    fi

    if [[ -n "$JOBS" ]]; then
        CONFIG+=" --jobs $JOBS"
    else
        # defaults to 4 jobs
        CONFIG+=" --jobs 4"
    fi

    for f in "$@"; do
        local NAME=$(basename -- "$f")
        local EXT="${NAME##*.}"
        local NAME="${NAME%.*}"
        if [[ -n "$DRYRUN" ]]; then
            echo waifu2x $CONFIG -i "$f" -o "$NAME$POSTFIX.$EXT"
        else
            waifu2x $CONFIG -i "$f" -o "$NAME$POSTFIX.$EXT"
        fi
    done
}

# enable autocd
shopt -s autocd

# flush config
alias flush-bashrc="source ~/.bashrc"
