#!/usr/bin/env elvish
use path

var script_name = (path:base (src)[name])

if (!= (count $args) 1) {
    echo Usage:
    echo "\t" $script_name input.mp4
    echo
    echo "\t" output will be like input.mp4.jpg
    exit
}

ffmpeg -i $args[0] ^
  -vf "select=eq(n\\,0),scale=w=in_w*sar:h=in_h" ^
  -vframes 1 ^
  $args[0].jpg