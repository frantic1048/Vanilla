#!/bin/env elvish

use path

var script_name = (path:base (src)[name])

if (!= (count $args) 0) {
    echo Usage:
    echo "\t" $script_name
    echo
    echo "\t" 'rename files like "12345_p0.jpg" to "pixiv 12345_p0.jpg" in curren folder.'
    exit
}

fd -e jpg -e png --type file -d 1 '^[0-9]+_p[0-9]+\.\w+$' -x mv '{/}' 'pixiv {/}'
