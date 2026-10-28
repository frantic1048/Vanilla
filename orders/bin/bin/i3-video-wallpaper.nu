#!/usr/bin/env nu

# Play a looping video as the wallpaper of one X11 output.
#
# Depends on:
#   xrandr
#   aur/xwinwrap-git, https://github.com/ujjwal96/xwinwrap
#   mpv

# Connected outputs and their geometry, e.g. DisplayPort-0 2560x1440+0+0.
def output-geometries []: nothing -> table<output: string, geometry: string> {
  # DisplayPort-0 connected primary 2560x1440+0+0 (normal left inverted right x axis y axis) 597mm x 336mm
  ^xrandr
  | lines
  | where $it =~ '\bconnected\b'
  | parse --regex '^(?<output>[\w-]+) [\w ]+?(?<geometry>\d+x\d+\+\d+\+\d+)\b'
}

def main [
  output?: string # xrandr output to cover.
  video?: path # Video file to play.
] {
  let geometries = (output-geometries)

  if $output == null or $video == null {
    let script_name = ($env.CURRENT_FILE | path basename)
    print --stderr $"Usage: ($script_name) <OUTPUT> <PATH_TO_VIDEO_FILE>"
    print --stderr ""
    print --stderr "OUTPUT (geometry):"
    for g in $geometries { print --stderr $"\t($g.output) \(($g.geometry)\)" }
    exit 1
  }

  let geometry = ($geometries | where output == $output | get geometry.0?)
  if $geometry == null {
    print --stderr $"Output \"($output)\" cannot be recognized, check xrandr output!"
    print --stderr "Supported outputs:"
    for g in $geometries { print --stderr $"\t($g.output)" }
    exit 1
  }

  # TODO: optimize video before using? (lower the cost of video wallpaper)
  # see https://gist.github.com/CSaratakij/788261f1ebcf2aefa320255120f75efe
  #
  # mpv costs:
  #   ~170M MEM, 2.5~6% CPU on 2950X desktop, h264 1920x1080 119.880fps
  (^xwinwrap -ov -g $geometry --
    mpv -wid WID
    --no-osc --no-osd-bar --no-input-default-bindings
    --loop --no-audio
    --hwdec
    $video)
}
