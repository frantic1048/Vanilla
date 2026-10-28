#!/usr/bin/env nu

# Remux videos to MP4 without re-encoding.
def main [
  ...videos: string # Videos to remux. Outputs are <name>.mp4 in the current directory.
] {
  if ($videos | is-empty) {
    let script_name = ($env.CURRENT_FILE | path basename)
    print --stderr $"Usage: ($script_name) video1.mkv video2.mkv ..."
    exit 1
  }

  for video in ($videos | where { is-not-empty }) {
    let parsed = ($video | path parse)
    if $parsed.extension == 'mp4' {
      print $"Skipping mp4 file: ($video)"
      continue
    }
    print $"Processing ($video)"
    ^ffmpeg -i $video -c copy $"($parsed.stem).mp4"
  }
}
