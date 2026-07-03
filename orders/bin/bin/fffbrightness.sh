#set brightness for intel HD videocard


BRIGHTNESS_DEFAULT=20 #Default brightness value
BRIGHTNESS_MAX=976
BRIGHTNESS_MIN=1
ROOT_UID=0
E_NOTROOT=87

if [ "$UID" -ne "$ROOT_UID" ]
then
    echo "Must be root to run this script."
    exit $E_NOTROOT
fi

if [ -n "$1" ]
then
    if [[ $1 =~ ^[0-9]+$ ]]
    then
        if [ $(($1)) -gt $BRIGHTNESS_MAX ]
        then
            brightness=$BRIGHTNESS_MAX
        elif [ $(($1)) -lt $BRIGHTNESS_MIN ]
        then
            brightness=$BRIGHTNESS_MIN
        else
            brightness=$(($1))
        fi
    else
        if [ "$1" == "max" ]
        then
            brightness=$BRIGHTNESS_MAX
        elif [ "$1" == "min" ]
        then
            brightness=$BRIGHTNESS_MIN
        else
            brightness=$BRIGHTNESS_DEFAULT
        fi
    fi
else
    brightness=$BRIGHTNESS_DEFAULT
fi

echo $brightness > /sys/class/backlight/intel_backlight/brightness
