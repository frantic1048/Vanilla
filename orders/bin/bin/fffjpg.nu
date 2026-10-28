#!/usr/bin/env nu

# Convert images to JPEG in parallel, shrinking anything larger than 2800x2800.
def main [
  ...images: string # Images to convert. Outputs are named <input>.jpg.
] {
  if ($images | is-empty) {
    let script_name = ($env.CURRENT_FILE | path basename)
    print --stderr $"Usage: ($script_name) image1.webp image2.png ..."
    exit 1
  }

  $images | where { is-not-empty } | par-each {|image|
    print $"converting ($image)"
    ^magick $image -resize '2800x2800>' -quality 95 $"($image).jpg"
  } | ignore
}
