#!/usr/bin/env elvish

# convert the format to mp4 without encoding

use path
use str

var script_name = (path:base (src)[name])

if (== (count $args) 0) {
    echo Usage:
    echo "\t" $script_name video1.mkv video2.mkv ...
    echo
    echo "\t" output will be like video1.mp4 video2.mp4 ...
    exit
}

put $@args | each {|video|
    if (!=s $video '') {
        echo Processing $video
        # -resize '2160x2160>' ^
        var file_name = (path:base $video)
        var ext_name = (path:ext $video)

        if (==s $ext_name '.mp4') {
            echo Skipping mp4 file: $video
        } else {
            var base_name = (str:trim-suffix $file_name $ext_name)
            var out_name = $base_name'.mp4'

            ffmpeg -i $video -c copy $out_name
        }
    }
}
