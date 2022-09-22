#!/usr/bin/env elvish

use whtsky
use path

var script_name = (path:base (src)[name])

fn usage {
    echo Usage:
    echo "\t" $script_name "<0|1|2|3|...>"
}

fn update_bar {
    if ?(search-external sketchybar) {
        sketchybar --update
    }
}

# n: (num 1.0)
fn float_to_int_str {|n|
    #put (printf "%.0f" $n)
    put (to-string (exact-num $n))
}

fn destroy_empty_offscreen_spaces {
    var spaces_list = (yabai -m query --spaces | from-json)
    var empty_offscreen_spaces_list = (whtsky:filter $spaces_list {|space|
        and (== (count $space[windows]) 0) (not $space[is-visible])
    })

    each {|space|
        yabai -m space --destroy (float_to_int_str $space[index])
    } $empty_offscreen_spaces_list
}

fn space_index_to_label {|space_index|
    if (==s (kind-of $space_index) "number") {
        # qaq
        put 'w'(float_to_int_str $space_index)
    } else {
        put 'w'$space_index
    }
}


# (create and)switch to space with label "w<target_space_index>"
# target_space_index should be an integer
fn switch_to_space {|target_space_index|
    var target_space_label = (space_index_to_label $target_space_index)
    var spaces_list = (yabai -m query --spaces | from-json)

    var target_space_exists = (whtsky:some $spaces_list {|space|
        ==s $space[label] $target_space_label
    })

    if $target_space_exists {
        yabai -m space --focus $target_space_label
    } else {
        yabai -m space --create
        var new_space_index = (exact-num (yabai -m query --spaces | from-json)[-1][index])
        var new_space_label = (space_index_to_label $new_space_index)
        yabai -m space $new_space_index --label $new_space_label
        yabai -m space --focus $new_space_label
    }

    destroy_empty_offscreen_spaces
}

# assign w1, w2, w3... labels to existing spaces which does not have a label
fn init_space_labels {
    var spaces_list = (yabai -m query --spaces | from-json)
    var existing_label_list = (whtsky:map $spaces_list {|space| ^
        if (!=s $space[label] "") {
            put $space[label]
        }
    })
    var index_for_label = 0

    fn get_next_label {
        set index_for_label = (+ $index_for_label 1)
        var _label = (space_index_to_label $index_for_label)
        if (not (whtsky:some $existing_label_list $_label)) {
            put $_label
            set existing_label_list = [$@existing_label_list $_label]
        } else {
            put (get_next_label)
        }
    }

    each {|space|
        if (==s $space[label] "") {
            yabai -m space (exact-num $space[index]) --label (get_next_label)
        }
    } $spaces_list
}

var options = [
    &flag_init_labels='--init-space-labels'
]

if (!= (count $args) 1) {
    usage
    exit
}

if (==s $args[0] $options[flag_init_labels]) {
    init_space_labels
    update_bar
} elif ?(exact-num $args[0]) {
    switch_to_space (exact-num $args[0])
    update_bar
} else {
    echo Unknown option, noting to do.
    usage
}