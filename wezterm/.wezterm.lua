local wezterm = require 'wezterm'

return {
    default_prog = { '/usr/local/bin/elvish' },

    -- light
    color_scheme = 'Spring',
    colors = {
        foreground = '#000000',
        background = '#eeefef'
    },

    font = wezterm.font 'JetBrains Mono',
    window_background_opacity = 0.8,
    enable_tab_bar = false,
    hide_tab_bar_if_only_one_tab = true
}
