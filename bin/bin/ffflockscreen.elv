#!/bin/env elvish

#spectacle -bnf -o /tmp/fff_screen_lock.png
scrot -zm /tmp/fff_screen_lock.png
i3lock -tue -p win -i /tmp/fff_screen_lock.png
