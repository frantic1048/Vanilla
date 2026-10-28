#!/usr/bin/env nu

# Personal aliases for interactive Nushell sessions.

# Shell-neutral git shortcuts.
export use ~/.local/bin/van/alias-g g

export alias b = ^bat
export alias c = clear
export alias ip = ^ip -c
export alias e = ^eza
export alias ee = ^eza -l
export alias tree = ^eza -T
export alias p = ^paru
export alias aria = ^aria2c $"--conf-path=($nu.home-dir)/bkped/aria2c.conf"
export alias s = ^systemctl
export alias f = ^fd
export alias r = ^rg
export alias i = ^time
export alias d = ^docker
export alias q = ^qalc
export alias j = ^jj

export def p-rm-orphan [] {
  ^paru -Rns ...(^paru -Qtdq | lines)
}

# workaround for macOS 26
# https://github.com/microsoft/vscode/pull/267724#issuecomment-3316457267
export def --wrapped code [...args] {
  with-env {CHROME_HEADLESS: 1} { ^code ...$args }
}

export def br [] {
  ^git for-each-ref 'refs/heads' '--format=%(color:cyan)%(refname:short)'
}

# TODO: better place for this
export def gsign_on [] {
  if ('.sl' | path type) == dir { ^sl config --local gpg.enabled true }
  if ('.git' | path type) == dir { ^git config commit.gpgsign true }
}
export def gsign_off [] {
  if ('.sl' | path type) == dir { ^sl config --local gpg.enabled false }
  if ('.git' | path type) == dir { ^git config commit.gpgsign false }
}

# disable annoying auto word wrap...
export alias nano = ^nano -w

export alias bat = ^bat --theme=TwoDark

export alias neofetch = ^fastfetch

export alias prpr = ^proxychains
export alias prprme = ^proxychains nu

# simple py http server
export alias pyserv = ^python -m http.server

# count files/matches of folder
export def count-file [...paths: string]: nothing -> int {
  ^find ...$paths -type f | lines | length
}
export def count-match [pattern: string]: nothing -> int {
  ^rg -ci --no-filename $pattern | lines | into int | math sum
}

# start hefur bittorrent tracker
export alias tracker = ^hefurd -ipv6 -log-color -log-level info -udp-port 6969 -http-port 6969 -https-port 6970

# get WAN IP address
export def ipwan []: nothing -> string {
  ^dig +short myip.opendns.com @resolver1.opendns.com | str trim
}
# get ipinfo(need prpr configured)
export def ipinfo [ip: string = ""] {
  ^proxychains curl --silent $"ipinfo.io/($ip)"
}
export def ipwaninfo [] { ipinfo (ipwan) }

# macOS only
export def renewip [] {
  ^networksetup -setbootp Ethernet
  ^networksetup -setdhcp Ethernet
}

# https://sw.kovidgoyal.net/kitty/conf/#kitty-conf
export alias kitty-reload = ^killall -SIGUSR1 kitty

export alias less-watch-file = ^less -R +F
