#!/bin/env elvish

use kokkoro

var at-env~ = $kokkoro:at-env~
var hosts = $kokkoro:hosts
var desktops = $kokkoro:desktops

at-env &host=$hosts[fantastic-rabbithouse] &desktop=$desktops[sway] {
    # left
    swaymsg output '"Dell Inc. DELL U2414H GN64V74A24AL"' ^
        transform 270 ^
        pos 0 0 ^
        bg {~}'/Pictures/photo/yande.re 482810.jpg' fill

    # center
    swaymsg output '"Dell Inc. DELL U2414H GN64V73N2WRL"' ^
        pos 1080 800 ^
        bg {~}'/Pictures/photo/yande.re 570936.png' fill

    # right
    swaymsg output '"Apple Computer Inc Color LCD 0x00000000"' ^
        transform 270 ^
        pos 3000 800 ^
        scale 2 ^
        bg {~}'/Pictures/photo/twitter EKNNutiUYAYcNzW.jpg' fill
}

at-env &host=$hosts[amausaan] &desktop=$desktops[i3] {
    xrandr --dpi 144
    xrandr --output DisplayPort-0 --mode 2560x1440 --rate 144
    xrandr --output DisplayPort-1 --mode 3840x2160 --rate 60
    xrandr --output DisplayPort-0 --left-of DisplayPort-1
    xrandr --output DisplayPort-0 --primary

    dispwin -d 1 -I {~}'/.local/share/DisplayCAL/storage/27GL850 #1 2021-06-05 15-34 sRGB F-S 3xCurve+MTX/27GL850 #1 2021-06-05 15-34 sRGB F-S 3xCurve+MTX.icc'
    dispwin -d 2 -I {~}'/.local/share/DisplayCAL/storage/Ultra HD #2 2020-07-01 23-25 2.2 F-S 3xCurve+MTX/Ultra HD #2 2020-07-01 23-25 2.2 F-S 3xCurve+MTX.icc'
    #setwallpaper -m fill {~}'/Pictures/bg/yande.re 432070.png'
    nitrogen --head=0 --set-zoom-fill {~}'/Pictures/bg/photo/_DSC3921.jpg'
    nitrogen --head=1 --set-zoom-fill {~}'/Pictures/bg/photo/_DSC4093.png'
}

at-env &host=$hosts[chimame-tai] &desktop=$desktops[i3] {
    xrandr --dpi 192

    xrandr --output DP-2 --rotate left
    xrandr --output DP-1 --rotate right
    xrandr --output eDP-1 --rotate right
    xrandr --output DP-1 --right-of DP-2
    xrandr --output eDP-1 --right-of DP-1

    dispwin -I {~}'/.local/share/DisplayCAL/storage/Monitor 1 #1 2019-10-07 01-40 D6500 2.2 F-S 3xCurve+MTX/Monitor 1 #1 2019-10-07 01-40 D6500 2.2 F-S 3xCurve+MTX.icc'
#    setwallpaper -m fill '/home/chino/Pictures/bg/kokkoro-princess_waifu_s1_n2.png'
    nitrogen --head=0 --set-zoom-fill {~}'/Pictures/bg/pixiv 82291538_p0.jpg'
    nitrogen --head=1 --set-zoom-fill {~}'/Pictures/bg/pixiv 83588446_p0.jpg'
    nitrogen --head=2 --set-zoom-fill {~}'/Pictures/bg/pixiv 83453696_p1.jpg'

    # auto rotate screen
    # pkg: iio-sensor-proxy aur/screenrotator-git
    if (not ?(pgrep screenrotator)) {
        screenrotator &
    }
}

at-env &host=$hosts[chimame-tai] &desktop=$desktops[sway] {

    var left_screen_offset_y = 235
    # left
    swaymsg output '"Dell Inc. DELL U2414H GN64V74A24AL"' ^
        transform 0 ^
        pos 0 $left_screen_offset_y ^
        bg {~}'/Pictures/bg/photo/_DSC3011.png' fill

    # center
    swaymsg output '"Dell Inc. DELL U2414H GN64V73N2WRL"' ^
        transform 90 ^
        pos 1920 0 ^
        bg {~}'/Pictures/bg/twitter EpLcMJYVoAAp0oo.jpg' fill

    # right
    swaymsg output '"Chimei Innolux Corporation 0x1373 0x00000000"' ^
        transform 90 ^
        pos 3000 680 ^
        scale 2 ^
        bg {~}'/Pictures/bg/pixiv 73124833_p0.png' fill

    #swaymsg output HEADLESS-1 ^
    #    pos 250 (+ 1080 $left_screen_offset_y) ^
    #    resolution 2800x1752 ^
    #    scale 2 ^
    #    bg {~}'/Pictures/bg/IMG_2950.png' fill

    # MEMO: auto rotate screen?
    # pkg: rot8
}


at-env &host=$hosts[toastx900] &desktop=$desktops[sway] {

    var var left_screen_offset_y = 235
    # left
    swaymsg output '"Dell Inc. DELL U2414H GN64V73N2WRL"' ^
        transform 90 ^
        pos 0 0 ^
        bg {~}'/Pictures/bg/pixiv 91108963_p0.jpg' fill
        # bg {~}'/Pictures/bg/photo/_DSC2171.png' fill

    # center
    swaymsg output '"Dell Inc. DELL U2414H GN64V74A24AL"' ^
        transform 0 ^
        pos 1080 600 ^
        bg {~}'/Pictures/bg/photo/_DSC3011.png' fill
        # bg {~}'/Pictures/bg/photo/_DSC2175.png' fill
        # bg {~}'/Pictures/bg/pixiv 86906320_p0.jpg' fill

    # HKC T4000
    # swaymsg output '"Unknown T4000+HDMI 0000000000001"' ^
    #    bg {~}'/Pictures/bg/pixiv 89478080_p1.jpg' fill

    #swaymsg output HEADLESS-1 ^
    var #    pos 250 (+ 1080 $left_screen_offset_y) ^
    #    resolution 2800x1752 ^
    #    scale 2 ^
    #    bg {~}'/Pictures/bg/IMG_2950.png' fill

    # MEMO: auto rotate screen?
    # pkg: rot8
}
