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
        bg {~}'/Pictures/photo/yande.re 482810.jpg' fill

    # center
    swaymsg output '"Dell Inc. DELL U2414H GN64V73N2WRL"' \
        pos 1080 800 \
        bg {~}'/Pictures/photo/yande.re 570936.png' fill

    # right
    swaymsg output '"Apple Computer Inc Color LCD 0x00000000"' \
        transform 270 \
        pos 3000 800 \
        scale 2 \
        bg {~}'/Pictures/photo/twitter EKNNutiUYAYcNzW.jpg' fill

    krunner -d --replace &
}

ON_HOST amausaan []{
    xrandr --dpi 144
    xrandr --output DisplayPort-0 --mode 2560x1440 --rate 144
    xrandr --output DisplayPort-1 --mode 3840x2160 --rate 60
    xrandr --output DisplayPort-0 --left-of DisplayPort-1
    xrandr --output DisplayPort-0 --primary

    dispwin -d 1 -I {~}'/.local/share/DisplayCAL/storage/27GL850 #1 2020-07-01 23-15 2.2 F-S 3xCurve+MTX/27GL850 #1 2020-07-01 23-15 2.2 F-S 3xCurve+MTX.icc'
    dispwin -d 2 -I {~}'/.local/share/DisplayCAL/storage/Ultra HD #2 2020-07-01 23-25 2.2 F-S 3xCurve+MTX/Ultra HD #2 2020-07-01 23-25 2.2 F-S 3xCurve+MTX.icc'
    #setwallpaper -m fill {~}'/Pictures/bg/yande.re 432070.png'
    nitrogen --head=0 --set-zoom-fill {~}'/Pictures/bg/(C97) [Naturefour (Mocha)] BUNHOUNYA3!.png'
    nitrogen --head=1 --set-zoom-fill {~}'/Pictures/bg/IMG_2614.png'
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
