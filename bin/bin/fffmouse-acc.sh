#!/bin/bash

#use parameter "on"  to turn on the mouse acceleration
#otherwise turn off the mouse acceleration

DeviceAccelProfile="libinput Accel Profile Enabled"
Acc="0, 1"
NoAcc="0, 1"

DeviceAccelConstantDeceleration="libinput Accel Speed"
AccSpeed=-0.2
NoAccSpeed=-0.2

#get from xinput
#sample output:
# ⎡ Virtual core pointer                          id=2    [master pointer  (3)]
# ⎜   ↳ RAPOO RAPOO 5G Wireless Device            id=11   [slave  pointer  (2)]
# ⎜   ↳ ETPS/2 Elantech Touchpad                  id=15   [slave  pointer  (2)]
# ⎜   ...
# ⎣ Virtual core keyboard                         id=3    [master keyboard (2)]
#     ...

#mouseName="RAPOO RAPOO 5G Wireless Device"
mouseName="SteelSeries SteelSeries Kinzu V3 Gaming Mouse"
mouse="pointer:${mouseName}"

if [[ -z `xinput | grep "$mouseName"".*pointer"` ]]
then
    echo 'No Mouse found ,Goshujinsama ×_×'
    exit
fi

if [ "$1" == "on" ]
then
    #an "on" parameter, turn on acceleration.
    xinput set-prop "${mouse}" "${DeviceAccelProfile}" $Acc
    xinput set-prop "${mouse}" "${DeviceAccelConstantDeceleration}" $AccSpeed
    echo 'Mouse Acceleration turned ON,Goshujinsama ^_^'
else
    #otherwise, turn off acceleration.
    xinput set-prop "${mouse}" "${DeviceAccelProfile}" $NoAcc
    xinput set-prop "${mouse}" "${DeviceAccelConstantDeceleration}" $NoAccSpeed
    echo 'Mouse Acceleration turned OFF,Goshujinsama ^.^'
fi
