#!/bin/env elvish

internal = eDP1
external = DP2

xrandr --newmode "1920x1200_60.00"  193.25  1920 2056 2256 2592  1200 1203 1209 1245 -hsync +vsync
xrandr --addmode $internal "1920x1200_60.00"
xrandr --output $internal --mode "1920x1200_60.00"
xrandr --output $external --right-of $internal

fffwallpaper.sh &

# need to restart conky
# to make it being correctly positioned
killall conky
conky -c /home/chino/.conky/i3bar/conkyrc.lua &
