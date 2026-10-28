#!/usr/bin/env nu

# Extract the first frame of a video as a JPEG, corrected for pixel aspect ratio.
def main [
  video?: string # Video to read. The output is <video>.jpg.
] {
  if $video == null {
    let script_name = ($env.CURRENT_FILE | path basename)
    print --stderr $"Usage: ($script_name) input.mp4"
    exit 1
  }

  (^ffmpeg -i $video
    -vf 'select=eq(n\,0),scale=w=in_w*sar:h=in_h'
    -vframes 1
    $"($video).jpg")
}
