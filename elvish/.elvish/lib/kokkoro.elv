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
    toastx900
])

desktops = (string-enum [
    i3
    sway
])

current-host = (uname -n)
current-desktop = $E:XDG_SESSION_DESKTOP

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
