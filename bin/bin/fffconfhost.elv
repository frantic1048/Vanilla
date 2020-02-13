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
    xrandr --dpi 192
    xrandr --output HDMI-A-0 --mode 2560x1440 --rate 120
    xrandr --output DisplayPort-0 --mode 3840x2160
    xrandr --output HDMI-A-0 --left-of DisplayPort-0

    dispwin -d 1 -I {~}'/.local/share/DisplayCAL/storage/Ultra HD #1 2019-10-07 00-12 2.2 F-S XYZLUT+MTX/Ultra HD #1 2019-10-07 00-12 2.2 F-S XYZLUT+MTX.icc'
    dispwin -d 2 -I {~}'/.local/share/DisplayCAL/storage/GS173QR #2 2020-02-06 09-55 2.2 M-S XYZLUT+MTX/GS173QR #2 2020-02-06 09-55 2.2 M-S XYZLUT+MTX.icc'
    setwallpaper -m fill {~}'/Pictures/bg/yande.re 432070.png'
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
