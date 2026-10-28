#!/usr/bin/env nu

# Print "artist - title" of the current media player every 2 seconds, for
# Waybar and tint2. Prints an empty line when nothing is playing.

def now-playing []: nothing -> string {
  # TODO: support player priority by name
  let player = (^playerctl -l | complete | get stdout | lines | get 0?)
  if $player == null { return "" }

  let artist = (^playerctl -p $player metadata artist | complete | get stdout | str trim)
  let title = (^playerctl -p $player metadata title | complete | get stdout | str trim)
  $"($artist) - ($title)"
}

def parent-pid []: nothing -> int {
  open --raw $"/proc/($nu.pid)/status"
  | lines
  | where $it starts-with 'PPid:'
  | first
  | split row --regex '\s+'
  | last
  | into int
}

def main [] {
  loop {
    print (now-playing)
    # Exit once orphaned, e.g. after the bar that started us is gone.
    if (parent-pid) == 1 { exit }
    sleep 2sec
  }
}
