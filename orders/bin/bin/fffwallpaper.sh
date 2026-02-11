#!/usr/bin/bash

LOCKFILE=/tmp/fffwallpaper.sh.lock

# kill previous instance
fuser -k $LOCKFILE

# run under lock
(
flock 200

PATH_LANDSCAPE=$(echo ~/sACG/_wallpaperLandscape)
PATH_PORTRAIT=$(echo ~/sACG/_wallpaperPortrait)
PATH_WALLPAPER=$PATH_LANDSCAPE

# in second
INTERVAL=300

help() {
    cat << EOF

Usage: fffwallpaper.sh [OPTIONS] FILE1 [FILE2 ...]

Options:
    -h               show this message.
    -l, --landscape  (default)use landscape wallpaper
    -p, --portrait   use portrait wallpaper
    -i <NUM>         (default 300)interval

EOF
}

while [[ $# -gt 0 ]]; do
    opt="$1"

    case $opt in
        -h)
            help
        ;;
        -l | --landscape)
            PATH_WALLPAPER=$PATH_LANDSCAPE
        ;;
        -p | --portrait)
            PATH_WALLPAPER=$PATH_PORTRAIT
        ;;
        -i)
            INTERVAL="$2"
            shift
        ;;
    esac
    shift
done


wlist=(${PATH_WALLPAPER}/*.{png,jpg})

while :
do
    # $RANDOM is reliable when $wlist is not very long
    w="${wlist[ $RANDOM % ${#wlist[@]}]}"
    echo $w
    feh --bg-fill "${w}"
    sleep $INTERVAL
done

)200>$LOCKFILE
