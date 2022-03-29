#!/bin/env elvish
use path
use re
use whtsky

# Depends on:
#   xrandr
#   aur/xwinwrap-git, https://github.com/ujjwal96/xwinwrap
#   mpv

var script_name = (path:base (src)[name])

# [&DisplayPort-0=	2560x1440+0+0]
var output_geometry_table = [&]

fn collect_output {
    # DisplayPort-0 connected primary 2560x1440+0+0 (normal left inverted right x axis y axis) 597mm x 336mm
    var lines = (whtsky:filter {|v| re:match '\bconnected\b' $v } [(xrandr)])

    # [[DisplayPort-0 0000x0000+0+0]]
    var monitors = (whtsky:map {|v|
        put (whtsky:map {|match|
                put $match[text]
            } (re:find '^([\w-]+) [\w ]+?(\d+x\d+\+\d+\+\d+)\b' $v)[groups][1..3]
        )
    } $lines)

    each {|m| set output_geometry_table[$m[0]] = $m[1]} $monitors
}

fn usage {
    echo Usage:
    echo "\t" $script_name "<OUTPUT> <PATH_TO_VIDEO_FILE>"
    echo
    echo OUTPUT: xrandr output name
}

if (!= (count $args) 2) {
    usage
    exit
}

collect_output

var output video = $@args

if (not (has-key $output_geometry_table $output)) {
    echo "Output \""$output"\" cannot be recognized, check xrandr output!"
    echo "Supported outputs:"
    each {|o| echo "\t"$o} [(keys $output_geometry_table)]
    fail "Output cannot be recognized"
}

# TODO: optimize video before using? (lower the cost of video wallpaper)
# see https://gist.github.com/CSaratakij/788261f1ebcf2aefa320255120f75efe
#
# mpv costs:
#   ~170M MEM, 2.5~6% CPU on 2950X desktop, h264 1920x1080 119.880fps

xwinwrap -ov -g $output_geometry_table[$output] -- ^
    mpv -wid WID ^
    --no-osc --no-osd-bar --no-input-default-bindings ^
    --loop --no-audio ^
    --hwdec ^
    $video
