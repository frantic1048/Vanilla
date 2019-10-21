#!/bin/env elvish

currentHost=(hostname)

fn ON_HOST [host config~]{
    if (==s $host $currentHost) {
        echo host:$host
        config
        exit
    }
}

ON_HOST fantastic-rabbithouse []{
    # left
    swaymsg output '"Dell Inc. DELL U2414H GN64V74A24AL"' \
        transform 270 \
        pos 0 0 \
        bg {~}'/Pictures/photo/yande.re 510907.jpg' fill

    # center
    swaymsg output '"Dell Inc. DELL U2414H GN64V73N2WRL"' \
        pos 1080 800 \
        bg {~}'/Pictures/photo/twitter DaqrTFdV4AAxPub_waifu_s4_n0.png.png' fill

    # right
    swaymsg output '"Apple Computer Inc Color LCD 0x00000000"' \
        transform 270 \
        pos 3000 800 \
        scale 2 \
        bg {~}'/Pictures/photo/twitter C_deiKLUwAA2C4I_waifu_s2_n1.jpg' fill

    krunner --replace &
}

ON_HOST amausaan []{
    xrandr --dpi 192
    dispwin -I '/home/chino/.local/share/DisplayCAL/storage/Ultra HD #1 2019-10-07 00-12 2.2 F-S XYZLUT+MTX/Ultra HD #1 2019-10-07 00-12 2.2 F-S XYZLUT+MTX.icc'
    setwallpaper -m fill {~}'/Pictures/bg/twitter EDYMd9fUEAAMkbC.jpg'
}

ON_HOST chimame-tai []{
    xrandr --dpi 192
    dispwin -I {~}'/.local/share/DisplayCAL/storage/Monitor 1 #1 2019-10-07 01-40 D6500 2.2 F-S 3xCurve+MTX/Monitor 1 #1 2019-10-07 01-40 D6500 2.2 F-S 3xCurve+MTX.icc'
    setwallpaper -m fill /home/chino/Pictures/bg/005.jpg.crop.tif
    
    # auto rotate screen
    # pkg: iio-sensor-proxy aur/screenrotator-git
    if (not ?(pgrep screenrotator)) {
        screenrotator &
    }
}
