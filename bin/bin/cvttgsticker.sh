#!/bin/bash
mkdir -p stickers
mogrify -resize 512x512\> -format png -path stickers *.png *.jpg
