#!/usr/bin/env nu

# Drop the audio track from videos without re-encoding.
def main [
  ...videos: string # Videos to strip. Outputs are <name>.na.<ext> in the current directory.
] {
  if ($videos | is-empty) {
    let script_name = ($env.CURRENT_FILE | path basename)
    print --stderr $"Usage: ($script_name) video1.mp4 video2.mp4 ..."
    exit 1
  }

  for video in ($videos | where { is-not-empty }) {
    let parsed = ($video | path parse)
    let output = if ($parsed.extension | is-empty) {
      $"($parsed.stem).na"
    } else {
      $"($parsed.stem).na.($parsed.extension)"
    }
    print $"Processing ($video)"
    ^ffmpeg -i $video -c copy -an $output
  }
}
