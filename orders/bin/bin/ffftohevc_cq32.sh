#!/bin/bash

# ffmpeg -i input -f mp4 -c:v hevc_nvenc -preset slow -qp 0 -c:a copy output.mp4
# ffmpeg -i input -f mp4 -c:v hevc_nvenc -preset slow -crf 16 -c:a copy output.mp4
# ffmpeg -i input -f mp4 -c:v hevc_nvenc -preset slow -cq 36 -c:a copy output.mp4

if [[ $# -eq 0 ]]; then
    cat << EOF

Usage: ffftohevc_cq32 input0 [input1...]

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

POSTFIX=".hevc.cq32"

for f in "$@"; do
    NAME=$(basename -- "$f")
    EXT="${NAME##*.}"
    NAME="${NAME%.*}"
    if [[ -n "$DRYRUN" ]]; then

        #echo ffmpeg -i "$f" -f mp4 -c:v hevc_nvenc -preset slow -cq 32 -c:a copy "$NAME$POSTFIX.mp4"
        echo ffmpeg -i "$f" -f mp4 -c:v hevc_nvenc -preset slow -cq 32 -c:a aac "$NAME$POSTFIX.mp4"
    else
        #ffmpeg -i "$f" -f mp4 -c:v hevc_nvenc -preset slow -cq 32 -c:a copy "$NAME$POSTFIX.mp4"
        ffmpeg -i "$f" -f mp4 -c:v hevc_nvenc -preset slow -cq 32 -c:a aac "$NAME$POSTFIX.mp4"
    fi
done
