#!/usr/bin/env elvish
use platform
use path

fn string-enum {|items-list|
    var result = [&]
    each {|item| set result[$item] = $item } $items-list

    put $result
}

var hosts = (string-enum [
    amausaan
    chimame-tai
    fantastic-rabbithouse
    toastx900
])

var desktops = (string-enum [
    i3
    sway
])

var current-host = (constantly (uname -n))
var current-desktop = $E:XDG_SESSION_DESKTOP

# linux,darwin
var current-os = $platform:os

fn at-env {|
    &host=''
    &desktop=''
    &os=''
    callback~
|
    if (or ^
        (and (!=s $host '') (!=s $host $current-host)) ^
        (and (!=s $desktop '') (!=s $desktop $current-desktop)) ^
        (and (!=s $os '') (!=s $os $current-os)) ^
    ) {
        return
    }

    callback
}

# return input string if it is a valid directory, otherwise return nothing
fn existing-dir { |maybe-dir| if (path:is-dir $maybe-dir) { put $maybe-dir } }
