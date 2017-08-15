#!/bin/env elvish

xrandr --newmode "1920x1200_60.00"  193.25  1920 2056 2256 2592  1200 1203 1209 1245 -hsync +vsync
xrandr --addmode eDP-1 "1920x1200_60.00"
xrandr --output eDP-1 --mode "1920x1200_60.00"

# need to restart conky
# to make it being correctly positioned
killall conky
conky -c /home/chino/.conky/i3bar/conkyrc.lua &
