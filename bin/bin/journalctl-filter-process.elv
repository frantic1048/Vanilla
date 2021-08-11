#!/bin/env elvish

use path

var script_name = (path:base (src)[name])

fn usage {
    echo Usage:
    echo journalctl -b0 '('$script_name processName0 processName1 ...')'
}

if (< (count $args) 1) {
    usage
    exit
}

# journalctl filter: https://serverfault.com/a/923436/449202
each [process_name]{ put (err = ?(pidof -S "\n" $process_name)) } $args | each [pid]{ echo '_PID='$pid }
