#!/bin/env elvish

internal = eDP1
external = DP2

# 1600x1000 59.87 Hz (CVT 1.60MA) hsync: 62.15 kHz; pclk: 132.25 MHz
# Modeline "1600x1000_60.00"  132.25  1600 1696 1864 2128  1000 1003 1009 1038 -hsync +vsync
# 1400x875 59.89 Hz (CVT) hsync: 54.38 kHz; pclk: 100.50 MHz
# Modeline "1400x875_60.00"  100.50  1400 1480 1624 1848  875 878 888 908 -hsync +vsync
#xrandr --newmode "1920x1200_60.00"  193.25  1920 2056 2256 2592  1200 1203 1209 1245 -hsync +vsync
#xrandr --addmode $internal "1920x1200_60.00"
#xrandr --output $internal --mode "1920x1200_60.00"

xrandr --newmode "1400x875_60.00"  100.50  1400 1480 1624 1848  875 878 888 908 -hsync +vsync
xrandr --addmode $internal "1400x875_60.00"
xrandr --output $internal --mode "1400x875_60.00"

#xrandr --output $external --mode "1920x1080"
#xrandr --output $external --right-of $internal

sleep 2
#fffwallpaper.sh &

# need to restart conky
# to make it being correctly positioned
killall conky
conky -c /home/chino/.conky/i3bar/conkyrc.lua &
