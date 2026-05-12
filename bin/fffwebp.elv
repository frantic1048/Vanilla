#!/usr/bin/env elvish

use path
use math
use flag
use str

var script-name = (path:base (src)[name])

# epoch time in seconds with fractional part
# requires GNU date
# FIXME: rewrite this script in other language with date time support later...
fn get-time {||
  put (num (date '+%s.%N'))
}

fn main {| &keeptime=$false &images=[] |
    var num-workers = (math:max 1 (math:floor (/ (nproc) 2)))

    var total-images = (count $images)
    var processed-images = 0

    # start time, seconds
    var start-time = (get-time)
    var last-processed-time-list = [$start-time]
    var max-processed-item-list-length = 16


    put $@images | each {|image|
        # skip non-file inputs
        if (!=s $image '') {
          put $image
        }
    } | peach &num-workers=$num-workers {|image|
          var speed = (if (> (count $last-processed-time-list) 1) {
              var time-diff = (- $last-processed-time-list[-1] $last-processed-time-list[0])
              var images-processed = (- (count $last-processed-time-list) 1)
              put (printf "[%.2f pic/s]" (/ $images-processed $time-diff))
          } else {
              put "[?_? pic/s]"
          })
          

          echo $speed' Converting '$image

          var output-image = $image'.webp'

          # 6144x6144 make sure whether portrait or landscape,
          # it will be large enough to fill a typical
          # 4k screen (3840x2160 or 3840x2400) without scaling up.
          #
          # 6144:3840 = 16:10
          # 3840:2400 = 16:10
          # 3840:2160 = 16:9
          var magick-args = [
            $image
            -resize '6144x6144>'
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

          set processed-images = (+ $processed-images 1)
          var time-list = (conj $last-processed-time-list (get-time))

          set last-processed-time-list = (if (> (count $time-list) $max-processed-item-list-length) {
              put $time-list[-$max-processed-item-list-length..]
          } else {
              put $time-list
          })
    }
  
    var end-time = (get-time)
    var total-time = (- $end-time $start-time)
    var speed-per-seconds = (if (> $total-time 0) {
        put (printf "%.2f" (/ $total-images $total-time))
    } else {
        put "O_O"
    })
    echo "["$speed-per-seconds' pic/s] All '$total-images' images converted.'
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
