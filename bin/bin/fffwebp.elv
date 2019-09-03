#!/bin/env elvish

script_name=(path-base (src)[name])

if (== (count $args) 0) {
    echo Usage:
    echo "\t" $script_name image1.png image2.jpg ...
    echo
    echo "\t" output will be like image1.png.webp image2.jpg.webp ...
    exit
}

put $@args | peach [image]{
    if (!=s $image '') {
        echo converting $image
        convert $image \
            -resize '2160x2160>' \
            -quality 95 \
            -define webp:method=6 \
            -define webp:auto-filter=true \
            -define webp:thread-level=1 \
            -define webp:pass=10 \
            $image'.webp'
    }
}
