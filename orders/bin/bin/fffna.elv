#!/usr/bin/env elvish

# drop the audio track from video files

use path
use str

var script_name = (path:base (src)[name])

if (== (count $args) 0) {
    echo Usage:
    echo "\t" $script_name video1.mp4 video2.mp4 ...
    echo
    echo "\t" output will be like video1.na.mp4 video2.na.mp4 ...
    exit
}

put $@args | each {|video|
    if (!=s $video '') {
        echo Processing $video
        # -resize '2160x2160>' ^
        var file_name = (path:base $video)
        var ext_name = (path:ext $video)
        var base_name = (str:trim-suffix $file_name $ext_name)
        var out_name = $base_name'.na'$ext_name

        ffmpeg -i $video -c copy -an $out_name
    }
}
