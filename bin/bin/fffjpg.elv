#!/bin/env elvish

script_name=(path-base (src)[name])

if (== (count $args) 0) {
    echo Usage:
    echo "\t" $script_name image1.webp image2.png ...
    echo
    echo "\t" output will be like image1.webp.jpg image2.png.jpg ...
    exit
}

put $@args | peach [image]{
    if (!=s $image '') {
        echo converting $image
        convert $image \
            -resize '2300x2300>' \
            -quality 93 \
            $image'.jpg'
    }
}
