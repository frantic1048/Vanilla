#!/usr/bin/env elvish

use path
use math
use flag

var script-name = (path:base (src)[name])

fn main {| &keeptime=$false &images=[] |
    var num-workers = (math:max 1 (math:floor (/ (nproc) 2)))

    put $@images | peach &num-workers=$num-workers {|image|
        if (!=s $image '') {
            echo converting $image

            var output-image = $image'.webp'

            var magick-args = [
              $image
              -resize '3840x3840>'
              -quality 95
              -define webp:method=6
              -define webp:auto-filter=true
              -define webp:thread-level=1
              -define webp:pass=10
              $output-image
            ]

            magick $@magick-args
            if $keeptime {
                touch -r $image $output-image
            }
        }
    }
}

var flag_specs = [
    [t $false "Keep original file's modification time"]
]
var parsed_args = [[&] []]
try {
  set parsed_args = [(flag:parse $args $flag_specs)]
} catch e {
  echo Exception: $e[reason]
}
var parsed_flags = $parsed_args[0]
var images = $parsed_args[1]

fn print-usage {||
    echo Usage:
    echo "\t" $script-name [OPTIONS] image1.png image2.jpg ...
    echo
    echo "\t" output will be like image1.png.webp image2.jpg.webp ...
    echo
    echo Options:
    echo "\t -t    keep original file's modification time"
}

if (== (count $images) 0) {
    print-usage
    exit 1
}

main &keeptime=$parsed_flags[t] &images=$images
