#!/usr/bin/env nu

# Per-host display layout and wallpaper, run by i3 and Sway on (re)load.

use kokkoro.nu

const home = $nu.home-dir

# Whether every given xrandr output is connected.
def xrandr-connected [...outputs: string]: nothing -> bool {
  let connected = (
    ^xrandr
    | lines
    | where $it =~ '\bconnected'
    | each { split row ' ' | first }
  )
  $outputs | all {|output| $output in $connected }
}

def main [] {
  if (kokkoro at-env --host fantastic-rabbithouse --desktop sway) {
    # left
    (^swaymsg output '"Dell Inc. DELL U2414H GN64V74A24AL"'
      transform 270
      pos 0 0
      bg $"($home)/Pictures/photo/yande.re 482810.jpg" fill)
    # center
    (^swaymsg output '"Dell Inc. DELL U2414H GN64V73N2WRL"'
      pos 1080 800
      bg $"($home)/Pictures/photo/yande.re 570936.png" fill)
    # right
    (^swaymsg output '"Apple Computer Inc Color LCD 0x00000000"'
      transform 270
      pos 3000 800
      scale 2
      bg $"($home)/Pictures/photo/twitter EKNNutiUYAYcNzW.jpg" fill)
  }

  if (kokkoro at-env --host amausaan --desktop i3) {
    print "setting dpi to 144"
    ^xrandr --dpi 144

    if (xrandr-connected "DisplayPort-0") {
      print "DisplayPort-0: setting mode and wallpaper"
      ^xrandr --output DisplayPort-0 --primary
      ^xrandr --output DisplayPort-0 --mode 2560x1440 --rate 144
      ^dispwin -d 1 -I $"($home)/.local/share/DisplayCAL/storage/27GL850_#1_2025-09-27_22-08_160cdm²_D6500_2.2_M-S_XYZLUT+MTX/27GL850_#1_2025-09-27_22-08_160cdm²_D6500_2.2_M-S_XYZLUT+MTX.icc"
      ^nitrogen --head=0 --set-zoom-fill $"($home)/Pictures/bg/photo/_DSC3936.jpg"
    } else {
      ^xrandr --output DisplayPort-0 --off
    }

    if (xrandr-connected "DisplayPort-1") {
      print "DisplayPort-1: setting mode and wallpaper"
      ^xrandr --output DisplayPort-1 --mode 2560x1440 --rate 60
      ^dispwin -d 2 -I $"($home)/.local/share/DisplayCAL/storage/SW270C_#2_2025-09-27_21-17_160cdm²_D6500_2.2_F-S_XYZLUT+MTX/SW270C_#2_2025-09-27_21-17_160cdm²_D6500_2.2_F-S_XYZLUT+MTX.icc"
      ^nitrogen --head=1 --set-zoom-fill $"($home)/Pictures/bg/photo/_DSC3936.jpg"
    } else {
      ^xrandr --output DisplayPort-1 --off
    }

    if (xrandr-connected "HDMI-A-0") {
      print "HDMI-A-0: setting mode and wallpaper"
      ^xrandr --output HDMI-A-0 --mode 1920x1080 --rate 60
      ^nitrogen --head=2 --set-zoom-fill $"($home)/Pictures/bg/photo/_DSC3936.jpg"
    } else {
      ^xrandr --output HDMI-A-0 --off
    }

    if (xrandr-connected "DisplayPort-0" "DisplayPort-1") {
      print "DisplayPort-0 and DisplayPort-1: setting position"
      ^xrandr --output DisplayPort-0 --above DisplayPort-1
    }
    if (xrandr-connected "DisplayPort-1" "HDMI-A-0") {
      print "DisplayPort-1 and HDMI-A-0: setting position"
      ^xrandr --output HDMI-A-0 --above DisplayPort-1
    }
  }

  if (kokkoro at-env --host amausaan --desktop sway) {
    # left
    (^swaymsg output '"Goldstar Company Ltd 27GL850 004NTWGBX241"'
      transform 0
      pos 0 0
      scale 1.4
      max_render_time 2
      bg $"($home)/Pictures/bg/photo/_DSC3936.jpg" fill)
    (^swaymsg output '"Goldstar Company Ltd LG Ultra HD 0x00009E6D"'
      transform 0
      pos 1829 0
      scale 2
      max_render_time 2
      bg $"($home)/Pictures/bg/photo/_DSC3936.jpg" fill)
  }

  if (kokkoro at-env --host chimame-tai --desktop i3) {
    ^xrandr --dpi 192
    ^xrandr --output DP-2 --rotate left
    ^xrandr --output DP-1 --rotate right
    ^xrandr --output eDP-1 --rotate right
    ^xrandr --output DP-1 --right-of DP-2
    ^xrandr --output eDP-1 --right-of DP-1
    ^dispwin -I $"($home)/.local/share/DisplayCAL/storage/Monitor 1 #1 2019-10-07 01-40 D6500 2.2 F-S 3xCurve+MTX/Monitor 1 #1 2019-10-07 01-40 D6500 2.2 F-S 3xCurve+MTX.icc"
    # ^setwallpaper -m fill '/home/chino/Pictures/bg/kokkoro-princess_waifu_s1_n2.png'
    ^nitrogen --head=0 --set-zoom-fill $"($home)/Pictures/bg/pixiv 82291538_p0.jpg"
    ^nitrogen --head=1 --set-zoom-fill $"($home)/Pictures/bg/pixiv 83588446_p0.jpg"
    ^nitrogen --head=2 --set-zoom-fill $"($home)/Pictures/bg/pixiv 83453696_p1.jpg"

    # auto rotate screen
    # pkg: iio-sensor-proxy aur/screenrotator-git
    if (^pgrep screenrotator | complete).exit_code != 0 {
      ^sh -c 'screenrotator >/dev/null 2>&1 &'
    }
  }

  if (kokkoro at-env --host chimame-tai --desktop sway) {
    let left_screen_offset_y = 235
    # left
    (^swaymsg output '"Dell Inc. DELL U2414H GN64V74A24AL"'
      transform 0
      pos 0 $left_screen_offset_y
      bg $"($home)/Pictures/bg/photo/_DSC3011.png" fill)
    # center
    (^swaymsg output '"Dell Inc. DELL U2414H GN64V73N2WRL"'
      transform 90
      pos 1920 0
      bg $"($home)/Pictures/bg/twitter EpLcMJYVoAAp0oo.jpg" fill)
    # right
    (^swaymsg output '"Chimei Innolux Corporation 0x1373 0x00000000"'
      transform 90
      pos 3000 680
      scale 2
      bg $"($home)/Pictures/bg/pixiv 73124833_p0.png" fill)
    # (^swaymsg output HEADLESS-1
    #   pos 250 (1080 + $left_screen_offset_y)
    #   resolution 2800x1752
    #   scale 2
    #   bg $"($home)/Pictures/bg/IMG_2950.png" fill)

    # MEMO: auto rotate screen?
    # pkg: rot8
  }

  if (kokkoro at-env --host toastx900 --desktop sway) {
    # left
    (^swaymsg output '"Dell Inc. DELL U2414H GN64V73N2WRL"'
      transform 90
      pos 0 0
      bg $"($home)/Pictures/bg/pixiv 91108963_p0.jpg" fill)
      # bg $"($home)/Pictures/bg/photo/_DSC2171.png" fill)
    # center
    (^swaymsg output '"Dell Inc. DELL U2414H GN64V74A24AL"'
      transform 0
      pos 1080 600
      bg $"($home)/Pictures/bg/photo/_DSC3011.png" fill)
      # bg $"($home)/Pictures/bg/photo/_DSC2175.png" fill)
      # bg $"($home)/Pictures/bg/pixiv 86906320_p0.jpg" fill)
    # HKC T4000
    # (^swaymsg output '"Unknown T4000+HDMI 0000000000001"'
    #   bg $"($home)/Pictures/bg/pixiv 89478080_p1.jpg" fill)
    # (^swaymsg output HEADLESS-1
    #   pos 250 (1080 + 235)
    #   resolution 2800x1752
    #   scale 2
    #   bg $"($home)/Pictures/bg/IMG_2950.png" fill)

    # MEMO: auto rotate screen?
    # pkg: rot8
  }
}
