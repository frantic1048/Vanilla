#!/usr/bin/env elvish

use re

#Mouse = 'Lenovo USB Receiver'
#Mouse = 'Logitech USB Receiver'
#Mouse = 'Logitech G Pro Gaming Mouse'
#Mouse = 'Logitech G102'
var Mouse = 'Logitech G903'

var DeviceAccelProfile = 'libinput Accel Profile Enabled'
var Acc = [1 0]
var NoAcc = [0 1]

var DeviceAccelConstantDeceleration = 'libinput Accel Speed'
var AccSpeed = '0'
# 1: Logitech G102
var NoAccSpeed = '1'

echo 'finding mouse: '$Mouse

var founded = $false

for device [(xinput)] {
    for pointer [(re:find $Mouse'.*id=([[:digit:]]+).*pointer' $device)] {
        var id = $pointer[groups][1][text]
        for prop [(xinput list-props $id)] {
            if (re:match $DeviceAccelProfile $prop) {
                # we have found the real mouse device
                echo 'found device (id='$id'): '$Mouse
                xinput set-prop $id $DeviceAccelProfile $@NoAcc
                xinput set-prop $id $DeviceAccelConstantDeceleration $NoAccSpeed
                set founded = $true
                break
            }
        }
    }
}

if $founded {
  echo '(っ*'ω'*c)﻿ mouse acc turned off.'
} else {
  echo 'ヾ(°ω｡ヽ=ﾉ°ω｡)ノ'$Mouse' not found.'
}
