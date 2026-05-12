#!/usr/bin/env nu

# Directory History Daemon
# attempt to replicate elvish's directory history feature

use std/log

def dhd_log [level msg] {
  # TODO: map level to log xxx calls
  log info $"[dhd] ($msg)"
}

def dhd [before after] {
  dhd_log debug $"moving from ($before) to ($after)"
}

export-env {
  $env.config.hooks.env_change = $env.config.hooks.env_change? | default {} | merge {
    PWD: ($env.config.hooks.env_change.PWD? | default [] | append {|before, after|
      dhd $before $after
    })
  }
}