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
        -i "eDP-1:/home/chino/Pictures/photo/twitter EKNNutiUYAYcNzW.jpg" \
        -i "DP-1:/home/chino/Pictures/photo/yande.re 570936.png" \
        -i "DP-2:/home/chino/Pictures/photo/yande.re 482810.jpg"
}

ON_HOST pending []{
    #spectacle -bnf -o /tmp/fff_screen_lock.png
    scrot -zmo /tmp/fff_screen_lock.png
    i3lock -tue -p win -i /tmp/fff_screen_lock.png
}

ON_HOST chimame-tai []{
    #scrot -zmo /tmp/fff_screen_lock.png
    #i3lock -tue -p win -i /tmp/fff_screen_lock.png
    #i3lock -tue -p win -i /home/chino/Pictures/bg/photo/IMG_2642_01.crop.4320x3840.png
    i3lock -tue -p win -i /home/chino/Pictures/bg/pcr/101431.2160x1920_waifu_s3_n0.bmp.png
}
