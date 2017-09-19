#!/bin/env elvish

Mouse = 'Lenovo USB Receiver'

DeviceAccelProfile = 'libinput Accel Profile Enabled'
Acc = [1 0]
NoAcc = [0 1]

DeviceAccelConstantDeceleration = 'libinput Accel Speed'
AccSpeed = '-0.2'
NoAccSpeed = '-0.2'

echo 'finding mouse: '$Mouse

founded = $false

for device [(xinput)] {
    for pointer [(re:find $Mouse'.*id=([[:digit:]]+).*pointer' $device)] {
        id = $pointer[groups][1][text]
        for prop [(xinput list-props $id)] {
            if (re:match $DeviceAccelProfile $prop) {
                # we have found the real mouse device
                echo 'found device (id='$id'): '$Mouse
                xinput set-prop $id $DeviceAccelProfile $@NoAcc
                xinput set-prop $id $DeviceAccelConstantDeceleration $NoAccSpeed
                founded = $true
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
