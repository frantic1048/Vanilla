local wezterm = require 'wezterm'

return {
    default_prog = {'/usr/local/bin/elvish'},

    -- light
    -- color_scheme = 'Spring',
    -- color_scheme = 'Tango',
    colors = {
        -- GitHub colors
        -- https://www.npmjs.com/package/@primer/primitives?activeTab=code
        -- find proper `ansi` key

        -- dark
        background = '#0a0a0a',
        foreground = '#cecfcf',
        ansi = {'#24292f', '#ff8182', '#4ac26b', "#d4a72c", "#54aeff", "#c297ff", "#76e3ea", '#d0d7de'},
        brights = {'#32383f', '#ffaba8', '#6fdd8b', "#eac54f", "#80ccff", "#d8b9ff", "#b3f0ff", '#d0d7de'}

        -- light
        -- background = '#fafbf9',
        -- foreground = '#3a3a3a',
        -- ansi = {"#24292f", "#cf222e", "#116329", "#4d2d00", "#0969da", "#8250df", "#1b7c83", "#6e7781"},
        -- brights = {"#57606a", "#a40e26", "#1a7f37", "#633c01", "#218bff", "#a475f9", "#3192aa", "#8c959f"}
    },

    -- font = wezterm.font 'JetBrains Mono',
    font = wezterm.font 'JetBrainsMono Nerd Font Mono',
    window_background_opacity = 0.64,
    enable_tab_bar = true,
    hide_tab_bar_if_only_one_tab = true
}
