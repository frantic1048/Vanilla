#!/usr/bin/env elvish

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

var current-host = (uname -n)
var current-desktop = $E:XDG_SESSION_DESKTOP

fn at-env {|
    &host=''
    &desktop=''
    callback~
|
    if (and (!=s $host '') (!=s $host $current-host)) {
        return
    }
    if (and (!=s $desktop '') (!=s $desktop $current-desktop)) {
        return
    }

    callback
}
