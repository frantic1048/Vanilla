#!/bin/env elvish

currentHost=(hostname)

fn ON_HOST [host config~]{
    if (==s $host $currentHost) {
        echo host:$host
        config
        exit
    }
}

ON_HOST amausaan []{
    xrandr --dpi 192
    dispwin -I '/home/chino/.local/share/DisplayCAL/storage/Ultra HD #1 2019-10-06 22-02 2.2 F-S XYZLUT+MTX/Ultra HD #1 2019-10-06 22-02 2.2 F-S XYZLUT+MTX.icc'
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
