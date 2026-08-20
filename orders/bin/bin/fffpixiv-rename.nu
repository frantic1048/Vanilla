#!/usr/bin/env nu

# Prefix Pixiv image filenames such as 12345_p0.jpg with "pixiv ".
def main [] {
  let files = (
    glob "*.{jpg,png}" --no-dir --no-symlink
    | where {|file|
      ($file | path basename) =~ '^[0-9]+_p[0-9]+\.(jpg|png)$'
    }
    | sort
  )

  for file in $files {
    let parsed = ($file | path parse)
    let destination = (
      [$parsed.parent $"pixiv ($parsed.stem).($parsed.extension)"]
      | path join
    )
    mv $file $destination
  }
}
