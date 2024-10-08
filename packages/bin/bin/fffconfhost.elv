#!/usr/bin/env elvish

use kokkoro
use whtsky
use str

var at-env~ = $kokkoro:at-env~
var hosts = $kokkoro:hosts
var desktops = $kokkoro:desktops

fn is_xrandr_display_connected {|@displays|
    var connected_displays = (make-map [(each {|display|
        put [ $display $false ]
    } $displays)])

    each {|connected_display|
        var matched_display = [(whtsky:find $displays {|display| str:contains $connected_display $display })]
        if (== (count $matched_display) 1) {
            set connected_displays[$matched_display[0]] = $true
        }
    } [(xrandr | rg '\bconnected')]

    whtsky:every $displays {|display| put $connected_displays[$display] }
}

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
    echo "setting dpi to 144"
    xrandr --dpi 144

    if (is_xrandr_display_connected "DisplayPort-0") {
        echo "DisplayPort-0: setting mode and wallpaper"
        xrandr --output DisplayPort-0 --primary
        xrandr --output DisplayPort-0 --mode 2560x1440 --rate 144
        dispwin -d 1 -I {~}'/.local/share/DisplayCAL/storage/27GL850 #1 2022-10-05 01-20 160cdm² D6500 2.2 F-S XYZLUT+MTX/27GL850 #1 2022-10-05 01-20 160cdm² D6500 2.2 F-S XYZLUT+MTX.icc'
        nitrogen --head=0 --set-zoom-fill {~}'/Pictures/bg/photo/_DSC3936.jpg'
    } else {
        xrandr --output DisplayPort-0 --off
    }

    if (is_xrandr_display_connected "DisplayPort-1") {
        echo "DisplayPort-1: setting mode and wallpaper"
        xrandr --output DisplayPort-1 --mode 2560x1440 --rate 60
        dispwin -d 2 -I {~}'/.local/share/DisplayCAL/storage/SW270C #2 2022-10-05 02-21 160cdm² D6500 2.2 F-S XYZLUT+MTX/SW270C #2 2022-10-05 02-21 160cdm² D6500 2.2 F-S XYZLUT+MTX.icc'
        nitrogen --head=1 --set-zoom-fill {~}'/Pictures/bg/photo/_DSC3936.jpg'
    } else {
        xrandr --output DisplayPort-1 --off
    }

    if (is_xrandr_display_connected "HDMI-A-0") {
        echo "HDMI-A-0: setting mode and wallpaper"
        xrandr --output HDMI-A-0 --mode 1920x1080 --rate 60
        nitrogen --head=2 --set-zoom-fill {~}'/Pictures/bg/photo/_DSC3936.jpg'
    } else {
        xrandr --output HDMI-A-0 --off
    }

    # MEMO: Cannot break multiple `and` arguments into multiple lines...
    if (is_xrandr_display_connected "DisplayPort-0" "DisplayPort-1") {
        echo "DisplayPort-0 and DisplayPort-1: setting position"
        xrandr --output DisplayPort-0 --left-of DisplayPort-1
    }

    if (is_xrandr_display_connected "DisplayPort-1" "HDMI-A-0") {
        echo "DisplayPort-1 and HDMI-A-0: setting position"
        xrandr --output HDMI-A-0 --above DisplayPort-1
    }
}

at-env &host=$hosts[amausaan] &desktop=$desktops[sway] {
        # left
    swaymsg output '"Goldstar Company Ltd 27GL850 004NTWGBX241"' ^
        transform 0 ^
        pos 0 0 ^
        scale 1.4 ^
        max_render_time 2 ^
        bg {~}'/Pictures/bg/photo/_DSC3936.jpg' fill

    swaymsg output '"Goldstar Company Ltd LG Ultra HD 0x00009E6D"' ^
        transform 0 ^
        pos 1829 0 ^
        scale 2 ^
        max_render_time 2 ^
        bg {~}'/Pictures/bg/photo/_DSC3936.jpg' fill

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
