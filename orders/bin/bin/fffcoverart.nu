#!/usr/bin/env nu

# Embed cover art into FLAC files.
#
# For each input file, look for cover.jpg or cover.png (any case) in the same
# directory and embed it. Files that are not FLAC, already have an embedded
# PICTURE block, or have no cover art next to them are skipped.
def main [
  ...files: string # FLAC files, e.g. (fd -e flac).
] {
  if ($files | is-empty) {
    let script_name = ($env.CURRENT_FILE | path basename)
    print --stderr $"Usage: ($script_name) a.flac [b.flac ...]"
    print --stderr $"       ($script_name) ...\(fd -e flac | lines\)"
    exit 1
  }

  for file in $files {
    let flac = ($file | path expand)

    if not ((^file -b $flac) starts-with 'FLAC audio bitstream data') {
      print $"Ignored, Non FLAC file: ($flac)"
      continue
    }
    if (^metaflac --list --block-type=PICTURE $flac | is-not-empty) {
      print $"Ignored, already have embedded cover: ($flac)"
      continue
    }

    let cover = (
      ls ($flac | path dirname)
      | where type == file and ($it.name | path basename) =~ '(?i)^cover\.(png|jpg)$'
      | get name.0?
    )
    if $cover == null {
      print $"Ignored, No cover art found: ($flac)"
      continue
    }

    ^metaflac $"--import-picture-from=($cover)" $flac
  }
}
