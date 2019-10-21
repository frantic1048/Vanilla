#!/bin/env elvish

currentHost=(hostname)

fn ON_HOST [host lock~]{
    if (==s $host $currentHost) {
        echo host:$host
        lock
        exit
    }
}

ON_HOST fantastic-rabbithouse []{
    swaylock -u \
        -i "DP-2:/home/chino/Pictures/photo/yande.re 510907.jpg" \
        -i "DP-1:/home/chino/Pictures/photo/twitter DaqrTFdV4AAxPub_waifu_s4_n0.png.png" \
        -i "eDP-1:/home/chino/Pictures/photo/twitter C_deiKLUwAA2C4I_waifu_s2_n1.jpg"
}

ON_HOST pending []{
    #spectacle -bnf -o /tmp/fff_screen_lock.png
    scrot -zm /tmp/fff_screen_lock.png
    i3lock -tue -p win -i /tmp/fff_screen_lock.png
}
