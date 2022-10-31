#!/usr/bin/env elvish

use whtsky
use path
use str

var script_name = (path:base (src)[name])

var options = [
    &init_space_labels='--init-space-labels'
    &switch_to_space='--switch-to-space'
    &init_sketchybar_spaces='--init-sketchybar-spaces'
    &update_sketchybar_space='--update-sketchybar-space'
]

fn usage {
    echo Usage:
    echo "Yabai space management:"
    echo "\t" $script_name $options[switch_to_space] "<0|1|2|3|...>"
    echo "\t" $script_name $options[init_space_labels]
    echo "Sketchybar:"
    echo "\t" $script_name $options[init_sketchybar_spaces]
    echo "\t" $script_name $options[update_sketchybar_space] "<0|1|2|3|...>"
}

fn update_bar {
    if (has-external sketchybar) {
        try {
            # sketchybar --update
        } catch e {
            # it happens when sketchybar is not running, or something is wrong
            # brew services restart sketchybar
            pprint $e
        }
    }
}

# n: (num 1.0) -> "1"
fn json_number_to_int_str {|n| put (to-string (exact-num $n)) }

fn destroy_empty_offscreen_spaces {
    var spaces_list = (yabai -m query --spaces | from-json)
    var empty_offscreen_spaces_list = (whtsky:filter $spaces_list {|space|
        and (== (count $space[windows]) 0) (not $space[is-visible])
    })

    each {|space|
        yabai -m space --destroy (json_number_to_int_str $space[index])
    } $empty_offscreen_spaces_list
}

fn space_index_to_label {|space_index|
    if (==s (kind-of $space_index) "number") {
        # $space_index could be (num 1.0), but we want (str "1")
        put 'w'(json_number_to_int_str $space_index)
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
        var new_space_index = (json_number_to_int_str (yabai -m query --spaces | from-json)[-1][index])
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
            yabai -m space (json_number_to_int_str $space[index]) --label (get_next_label)
        }
    } $spaces_list
}

fn update_sketchybar_space {|space_id|
    # space_id 1,2,3... matches spaces with label w1,w2,w3..., not space index
    var space_label = (space_index_to_label $space_id)
    var associated_space = [whtsky:find {|space|
        put (==s $space[label] $space_label)
    } (yabai -m query --spaces | from-json)]
    if (== (count $associated_space) 1) {
        var space = $associated_space[0]
        var background_drawing = ({ if $space[is-visible] { put 'on' } else { put 'off' } })
        sketchybar --set yabai_space.$space_id ^
            background.drawing=$background_drawing   ^
    } else {
        # hide this space indicator?
    }
}

fn init_sketchybar_spaces {
    var space_id_list = ['1' '2' '3' '4' '5' '6' '7' '8' '9' '10']
    each {|space_id|
        # var unprefixed_space_label = (str:trim-prefix $space_label 'w')
        sketchybar ^
            --add item yabai_space.$space_id ^
            --set yabai_space.$space_id ^
                update_freq=1 ^
                icon=$space_id                                      ^
                icon.padding_left=8                                 ^
                icon.padding_right=8                                ^
                background.padding_left=5                           ^
                background.padding_right=5                          ^
                background.color=0x44ffffff                         ^
                background.corner_radius=5                          ^
                background.height=22                                ^
                background.drawing=off                              ^
                label=""                                            ^
                label.drawing=off                                   ^
                script="~/bin/yabaictl.elv --update-sketchybar-space "$space_id ^
                click_script="~/bin/yabaictl.elv --switch-to-space "$space_id

        # --add space requires a space id in mission control
        # we're creating dynamic spaces, thus this is not usable
        # sketchybar ^
        #     --add space space.$space_id left                    ^
        #     --set  space.$space_id                                  ^
        #         icon=$space_id                                      ^
        #         icon.padding_left=8                                 ^
        #         icon.padding_right=8                                ^
        #         background.padding_left=5                           ^
        #         background.padding_right=5                          ^
        #         background.color=0x44ffffff                         ^
        #         background.corner_radius=5                          ^
        #         background.height=22                                ^
        #         background.drawing=off                              ^
        #         label=""                                            ^
        #         label.drawing=off                                   ^
        #         script="~/bin/yabaictl.elv --update-sketchybar-space "$space_id ^
        #         click_script="~/bin/yabaictl.elv --switch-to-space "$space_id
    } $space_id_list
}

fn cli {
    if (< (count $args) 1) {
        usage
        exit
    }

    if (==s $args[0] $options[init_space_labels]) {
        init_space_labels
        update_bar
    } elif (==s $args[0] $options[init_sketchybar_spaces]) {
        init_sketchybar_spaces
    } elif (and (==s $args[0] $options[update_sketchybar_space]) ?(json_number_to_int_str $args[1])) {
        update_sketchybar_space $args[1]
    } elif (and (==s $args[0] $options[switch_to_space]) ?(json_number_to_int_str $args[1])) {
        switch_to_space (json_number_to_int_str $args[1])
        update_bar
    } else {
        echo Unknown option, noting to do.
        usage
    }
}

cli
