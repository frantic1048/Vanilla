#!/bin/bash

# convert jpg/png images under cwd into ./stickers folder
# optimize for publishing telegram stickers
#
# requires:
#     - imagemagick: scaling and converting format
#     - optipng: optimize png size

mkdir -p stickers
mogrify -resize 512x512\> -format png -path stickers *.png *.jpg
optipng stickers/*.png
