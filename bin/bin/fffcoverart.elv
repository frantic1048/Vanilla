#!/bin/env elvish
use re

script_name=(path-base (src)[name])

exceptions=[
    &input_not_flac='Ignored, Non FLAC file'
    &no_cover_found='Ignored, No cover art found'
    &already_have_cover='Ignored, already have embedded cover'
]

fn usage []{
    echo Usage:
    echo "\t" $script_name a.flac [b.flac ...]
    echo "\t" $script_name '(fd -e flac)'
    echo
    echo "For each input file:"
    echo "\t"$script_name" will find cover art named cover.jpg|png in"
    echo "\tsame folder where each file is, and try to embed the cover art"
    echo "\tinto each flac file."
    echo
    echo "\tThis script will do nothing if no cover art found,"
    echo "\tor flac file already have one PICTURE in metadata"
}

fn assert_valid_flac [f]{
    if (not (re:match '^FLAC audio bitstream data' (file -b $f))) {
        fail $exceptions[input_not_flac]":"$f
    }
}

fn find_coverart [f]{
    search_dir = (path-dir $f)
    coverart_list = [(fd \
        --absolute-path \
        --type file \
        --max-depth 1 \
        --ignore-case \
        '^cover\.(png|jpg)$' \
        $search_dir
    )]

    if (> (count $coverart_list) 0) {
        # use the first match as cover art
        put $coverart_list[0]
    } else {
        fail $exceptions[no_cover_found]":"$f
    }
}

fn assert_flac_no_embedded_cover [f]{
    picture=[(metaflac --list --block-type=PICTURE $f)]
    if (!= 0 (count $picture)) {
        fail $exceptions[already_have_cover]':'$f
    }
}

if (== (count $args) 0) {
    usage
    exit
}

put $@args | each [file]{
    flac_file = (path-abs $file)

    try {
        assert_valid_flac $flac_file
        assert_flac_no_embedded_cover $flac_file
    } except e {
        echo $e
        continue
    }

    coverart_file = $false

    try { coverart_file=(find_coverart $flac_file) } except e {
        echo $e
        continue
    }

    metaflac --import-picture-from=$coverart_file $flac_file
}