#!/usr/bin/env nu

# Turn off pointer acceleration for every attached mouse on X11 (libinput).
# Sway does the same natively with `input type:pointer` in its config.

# libinput Accel Speed (-1..1) for mice without an override; matches Sway's
# pointer_accel.
const default_speed = 1.0

# Per-mouse speed overrides, matched as substrings of the xinput device name.
const speed_overrides: list<record<name: string, speed: float>> = [
  # {name: 'Logitech G903', speed: 1.0}
]

const profile_prop = 'libinput Accel Profile Enabled'
const speed_prop = 'libinput Accel Speed'
# Touchpads also expose the acceleration properties; leave them alone.
const touchpad_prop = 'libinput Tapping Enabled'

def pointer-devices []: nothing -> table<id: string, name: string> {
  ^xinput list
  | lines
  | parse --regex '↳ (?<name>.+?)\s+id=(?<id>\d+)\s+\[slave\s+pointer'
  | select id name
}

# Property name -> list of values, from `xinput list-props`.
def device-props [id: string]: nothing -> record {
  ^xinput list-props $id
  | lines
  | parse --regex '^\s+(?<prop>.+?) \(\d+\):\s*(?<values>.*)$'
  | reduce --fold {} {|row, acc| $acc | insert $row.prop ($row.values | split row ', ') }
}

def speed-for [name: string]: nothing -> float {
  let override = ($speed_overrides | where {|o| $name | str contains $o.name } | get speed.0?)
  $override | default $default_speed
}

def main [] {
  let mice = (
    pointer-devices
    | insert props {|d| device-props $d.id }
    | where {|d| $profile_prop in $d.props and $touchpad_prop not-in $d.props }
  )

  for mouse in $mice {
    # The profile property is a one-hot vector: adaptive, flat[, custom].
    let flat = ($mouse.props | get $profile_prop | enumerate | each {|p| if $p.index == 1 { 1 } else { 0 } })
    let speed = (speed-for $mouse.name)
    ^xinput set-prop $mouse.id $profile_prop ...($flat | each { into string })
    ^xinput set-prop $mouse.id $speed_prop ($speed | into string)
    print $"flat profile, speed ($speed): ($mouse.name) \(id=($mouse.id)\)"
  }

  if ($mice | is-empty) {
    print 'ヾ(°ω｡ヽ=ﾉ°ω｡)ノ no mouse found.'
  } else {
    print "(っ*'ω'*c) mouse acc turned off."
  }
}
