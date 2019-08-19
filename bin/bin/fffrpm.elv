#!/bin/env elvish

while (put $true) {
    #echo (joins '/' [[(sensors applesmc-isa-0300 | rg '^Exhaust {2}:' | rg -o '[[:digit:]]{1,4}')][0 2]]) 'RPM'
    echo [(sensors applesmc-isa-0300 | rg '^Exhaust {2}:' | rg -o '[[:digit:]]{1,4}')][0] 'RPM'
    sleep 2
}
