#!/usr/bin/env nu

def ffconcat-entry [file: path]: nothing -> string {
  let quote = (char --integer 39)
  let escaped_quote = $"($quote)\\($quote)($quote)"
  let escaped_path = (
    $file
    | path expand
    | into string
    | str replace --all $quote $escaped_quote
  )
  $"file ($quote)($escaped_path)($quote)"
}

def default-output [first_file: path]: nothing -> path {
  let stem = ($first_file | path parse | get stem)
  let common_stem = ($stem | str replace --regex '[0-9]{2}$' '')
  $"($common_stem)_full.wmv"
}

# Concatenate numbered WMV segments without re-encoding.
def main [
  --output (-o): path # Output file. Defaults to <common-prefix>_full.wmv.
  ...inputs: path # Ordered input files; discovers **/*[0-9][0-9].wmv when omitted.
] {
  let files = if ($inputs | is-empty) {
    glob "**/*[0-9][0-9].wmv" | sort
  } else {
    $inputs
  }

  if ($files | is-empty) {
    print --stderr "No WMV segments matching **/*[0-9][0-9].wmv were found."
    exit 1
  }

  let output_file = if $output == null {
    default-output $files.0
  } else {
    $output
  }

  print "Files to concatenate:"
  for file in $files {
    print $"  ($file)"
  }
  print $"Output: ($output_file)"

  let manifest_file = (mktemp --suffix .ffconcat)
  let manifest = (
    $files
    | each {|file| ffconcat-entry $file }
    | str join (char newline)
  )
  $"($manifest)(char newline)" | save --force $manifest_file

  mut ffmpeg_exit_code = 0
  try {
    ^ffmpeg -f concat -safe 0 -i $manifest_file -c copy $output_file
    $ffmpeg_exit_code = $env.LAST_EXIT_CODE
  } finally {
    rm --force $manifest_file
  }

  if $ffmpeg_exit_code != 0 {
    exit $ffmpeg_exit_code
  }
}
