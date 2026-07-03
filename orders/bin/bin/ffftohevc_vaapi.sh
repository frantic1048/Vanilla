#!/bin/bash

# https://trac.ffmpeg.org/wiki/Hardware/VAAPI
# ffmpeg -hwaccel vaapi -hwaccel_output_format vaapi -hwaccel_device /dev/dri/renderD128 -i 'input.wmv' -c:v hevc_vaapi output.mp4

if [[ $# -eq 0 ]]; then
    cat << EOF

Usage: ffftohevc_vaapi input0 [input1...]

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

POSTFIX="_hevc"

for f in "$@"; do
    NAME=$(basename -- "$f")
    EXT="${NAME##*.}"
    NAME="${NAME%.*}"
    if [[ -n "$DRYRUN" ]]; then
        echo ffmpeg -hwaccel vaapi -hwaccel_output_format vaapi -hwaccel_device /dev/dri/renderD128 \
            -i "$f" -c:v hevc_vaapi -c:a aac \
            "$NAME$POSTFIX.mp4"
    else
        ffmpeg -hwaccel vaapi -hwaccel_output_format vaapi -hwaccel_device /dev/dri/renderD128 \
            -i "$f" -c:v hevc_vaapi -c:a aac \
            "$NAME$POSTFIX.mp4"
    fi
done
