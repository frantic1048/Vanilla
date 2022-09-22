local wezterm = require 'wezterm'

return {
    default_prog = { '/usr/local/bin/elvish' },
    color_scheme = 'zenwritten_dark',
    font = wezterm.font 'JetBrains Mono',
    window_background_opacity = 0.4,
    enable_tab_bar = false,
    hide_tab_bar_if_only_one_tab = true
}
