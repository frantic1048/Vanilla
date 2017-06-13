#!/bin/bash

if [[ $# -eq 0 ]]; then
    cat << EOF
Convert pictures to single frame video

Usage: fffp2v input0 [input1...]

Options:
    -d          dry run, just print out the commands to exec.
    --          terminate options list

EOF
    exit 0
fi

DRYRUN=''

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
        *) # no more options
            break
        ;;
    esac
    shift
done

if [[ $# -eq 0 ]]; then
    echo no input file was specified, exiting.
    exit 1
fi

for f in "$@"; do
    NAME=$(basename -- "$f")
    EXT="${NAME##*.}"
    NAME="${NAME%.*}"
    if [[ -n "$DRYRUN" ]]; then
        echo ffmpeg -i "$f" -vf "scale=trunc(iw/2)*2:trunc(ih/2)*2" -pix_fmt yuv420p "$NAME.mp4"
    else
        ffmpeg -i "$f" -vf "scale=trunc(iw/2)*2:trunc(ih/2)*2" -pix_fmt yuv420p "$NAME.mp4"
    fi
done
