#!/bin/env elvish

fn select_player {
    players = [(playerctl -l)]
    # TODO: support player priority by name
    if (count $players) {
        put $players[0]
    }
}

fn beat {
    player = (select_player)
    if (== (count [player]) 0) {
        return
    }

    fn p [@args]{playerctl -p $player $@args}

    # TODO: parse (p metadata)
    # TODO: calculatet relative position
    artist = (p metadata artist)
    title = (p metadata title)
    status = (p status)
    echo $artist - $title
}

fn exit_when_parent_is_init {
    ppid = (cat /proc/$pid/status | rg '^PPid' | rg -o '[[:digit:]]+')
    if (== $ppid 1) {
        exit
    }
}

while (put $true) {
    beat
    exit_when_parent_is_init
    sleep 2
}
