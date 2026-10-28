#!/usr/bin/env nu

# Lock the screen with per-host wallpapers.

use kokkoro.nu

def main [] {
  if (kokkoro at-env --host fantastic-rabbithouse --desktop sway) {
    (^swaylock -u
      -i "eDP-1:/home/chino/Pictures/photo/twitter EKNNutiUYAYcNzW.jpg"
      -i "DP-1:/home/chino/Pictures/photo/yande.re 570936.png"
      -i "DP-2:/home/chino/Pictures/photo/yande.re 482810.jpg")
  } else if (kokkoro at-env --host chimame-tai --desktop i3) {
    # ^scrot -zmo /tmp/fff_screen_lock.png
    # ^i3lock -tue -p win -i /tmp/fff_screen_lock.png
    # ^i3lock -tue -p win -i /home/chino/Pictures/bg/photo/IMG_2642_01.crop.4320x3840.png
    # ^i3lock -tue -p win -i /home/chino/Pictures/bg/pcr/101431.2160x1920_waifu_s3_n0.bmp.png
    ^i3lock -tue -p win -i "/home/chino/Pictures/bg/fff_screen_lock.1.png"
  } else if (kokkoro at-env --host chimame-tai --desktop sway) {
    (^swaylock -u
      -i "eDP-1:/home/chino/Pictures/bg/pixiv 73124833_p0.png"
      -i "DP-1:/home/chino/Pictures/bg/twitter EpLcMJYVoAAp0oo.jpg"
      -i "DP-2:/home/chino/Pictures/bg/pixiv 85946505_p0.jpg")
      # -i "HEADLESS-1:/home/chino/Pictures/bg/IMG_2950.png"
  } else if (kokkoro at-env --host toastx900 --desktop sway) {
    (^swaylock -u
      -i "DP-1:/home/chino/Pictures/bg/photo/_DSC2075.png"
      -i "DP-2:/home/chino/Pictures/bg/photo/_DSC3011.png")
      # -i "HEADLESS-1:/home/chino/Pictures/bg/IMG_2950.png"
  }
}
