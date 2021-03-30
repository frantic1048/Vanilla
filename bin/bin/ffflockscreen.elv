#!/bin/env elvish

use kokkoro

at-env~ = $kokkoro:at-env~
hosts = $kokkoro:hosts
desktops = $kokkoro:desktops

at-env &host=$hosts[fantastic-rabbithouse] &desktop=$desktops[sway] {
    swaylock -u ^
            -i "eDP-1:/home/chino/Pictures/photo/twitter EKNNutiUYAYcNzW.jpg" ^
            -i "DP-1:/home/chino/Pictures/photo/yande.re 570936.png" ^
            -i "DP-2:/home/chino/Pictures/photo/yande.re 482810.jpg"
    exit
}

at-env &host=$hosts[chimame-tai] &desktop=$desktops[i3] {
#    scrot -zmo /tmp/fff_screen_lock.png
#    i3lock -tue -p win -i /tmp/fff_screen_lock.png
#    i3lock -tue -p win -i /home/chino/Pictures/bg/photo/IMG_2642_01.crop.4320x3840.png
#    i3lock -tue -p win -i /home/chino/Pictures/bg/pcr/101431.2160x1920_waifu_s3_n0.bmp.png
    i3lock -tue -p win -i "/home/chino/Pictures/bg/fff_screen_lock.1.png"
    exit
}

at-env &host=$hosts[chimame-tai] &desktop=$desktops[sway] {
    swaylock -u ^
        -i "eDP-1:/home/chino/Pictures/bg/pixiv 73124833_p0.png" ^
        -i "DP-1:/home/chino/Pictures/bg/twitter EpLcMJYVoAAp0oo.jpg" ^
        -i "DP-2:/home/chino/Pictures/bg/pixiv 85946505_p0.jpg"
        #-i "HEADLESS-1:/home/chino/Pictures/bg/IMG_2950.png"
    exit
}

at-env &host=$hosts[toastx900] &desktop=$desktops[sway] {
    swaylock -u ^
        -i "DP-2:/home/chino/Pictures/bg/twitter EpLcMJYVoAAp0oo.jpg" ^
        -i "DP-1:/home/chino/Pictures/bg/pixiv 85946505_p0.jpg"
        #-i "HEADLESS-1:/home/chino/Pictures/bg/IMG_2950.png"
    exit
}
