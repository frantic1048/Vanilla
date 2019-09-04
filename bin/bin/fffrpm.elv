#!/bin/env elvish

beat~ = []{
    echo [(sensors applesmc-isa-0300 | rg '^Exhaust {2}:' | rg -o '[[:digit:]]{1,4}')][0] 'RPM'
}

ppid~ = []{ put (cat /proc/$pid/status | rg '^PPid' | rg -o '[[:digit:]]+') }

while (put $true) {
    beat

    if (== (ppid) 1) {
        exit
    }

    sleep 2
}
