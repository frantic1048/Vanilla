#!/usr/bin/env elvish

fn string-enum [items-list]{
    result = [&]
    each [item]{ result[$item] = $item } $items-list

    put $result
}

hosts = (string-enum [
    amausaan
    chimame-tai
    fantastic-rabbithouse
])

desktops = (string-enum [
    i3
    sway
])

current-host = (hostname)
current-desktop = $E:XDG_CURRENT_DESKTOP


fn at-env [
    &host=''
    &desktop=''
    callback~
]{
    if (and (!=s $host '') (!=s $host $current-host)) {
        return
    }
    if (and (!=s $desktop '') (!=s $desktop $current-desktop)) {
        return
    }

    callback
}