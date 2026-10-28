#!/usr/bin/env nu

# Host and desktop facts for per-machine scripts.
#
#   use kokkoro.nu
#   if (kokkoro at-env --host amausaan --desktop i3) { ... }

export const hosts = [
  amausaan
  chimame-tai
  fantastic-rabbithouse
  toastx900
]

export const desktops = [
  i3
  sway
]

export def current-host []: nothing -> string {
  ^uname -n | str trim
}

export def current-desktop []: nothing -> string {
  $env.XDG_SESSION_DESKTOP? | default ""
}

# Whether this machine matches every given condition. Omitted conditions match
# anything. Unknown host or desktop names are errors, so a typo cannot silently
# never match.
export def at-env [
  --host: string # One of `kokkoro hosts`.
  --desktop: string # One of `kokkoro desktops`.
  --os: string # `$nu.os-info.name`, e.g. linux or macos.
]: nothing -> bool {
  if $host != null and $host not-in $hosts {
    error make {msg: $"kokkoro: unknown host '($host)'"}
  }
  if $desktop != null and $desktop not-in $desktops {
    error make {msg: $"kokkoro: unknown desktop '($desktop)'"}
  }
  (
    ($host == null or $host == (current-host))
    and ($desktop == null or $desktop == (current-desktop))
    and ($os == null or $os == $nu.os-info.name)
  )
}
