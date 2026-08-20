#!/usr/bin/env nu

const vaapi_device = "/dev/dri/renderD128"

def preset-args [preset: string]: nothing -> record<video: list<string>, audio: list<string>> {
  match $preset {
    "default" => {
      video: []
      audio: []
    }
    "old-film" => {
      video: [-rc_mode CQP -qp "20"]
      audio: [-af "aresample=async=1:first_pts=0"]
    }
    _ => {
      print --stderr $"Unknown preset '($preset)'. Expected 'default' or 'old-film'."
      exit 2
    }
  }
}

def output-path [input: path]: nothing -> path {
  let parsed = ($input | path parse)
  [$parsed.parent $"($parsed.stem)_hevc.mp4"] | path join
}

def display-command [args: list<string>]: nothing -> string {
  let displayed_args = (
    $args
    | each {|arg| $arg | into string | to nuon }
    | str join " "
  )
  $"^ffmpeg ($displayed_args)"
}

# Transcode videos to 10-bit HEVC with AMD VAAPI hardware encoding.
def main [
  --dry-run (-d) # Print each ffmpeg command without running it.
  --preset (-p): string = "default" # Encoding preset: default or old-film.
  ...inputs: path # Input videos. Outputs are written beside each input.
] {
  if ($inputs | is-empty) {
    print --stderr "No input file was specified."
    exit 1
  }

  let extra = (preset-args $preset)

  for input in $inputs {
    let args = [
      -init_hw_device $"vaapi=va:($vaapi_device)"
      -filter_hw_device va
      -i ($input | into string)
      -vf "format=p010,hwupload"
      -c:v hevc_vaapi
      -profile:v main10
      ...$extra.video
      -c:a aac
      ...$extra.audio
      (output-path $input | into string)
    ]

    if $dry_run {
      print (display-command $args)
    } else {
      ^ffmpeg ...$args
      if $env.LAST_EXIT_CODE != 0 {
        exit $env.LAST_EXIT_CODE
      }
    }
  }
}
