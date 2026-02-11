-- See https://wezfurlong.org/wezterm/
-- Add config folder to watchlist for config reloads.
local wezterm = require 'wezterm';
wezterm.add_to_config_reload_watch_list(wezterm.config_dir)

local wezterm = require 'wezterm'

return {
    -- FIXME: macOS homebrew x86_64 and aarch64 have different paths
    -- /usr/local/bin/elvish -- x86_64
    -- /opt/homebrew/bin/elvish -- aarch64
    -- default_prog = {'/usr/local/bin/elvish'},

    -- light
    -- color_scheme = 'Spring',
    -- color_scheme = 'Tango',
    color_scheme = 'Tokyo Night',
    colors = {
        -- GitHub colors
        -- https://www.npmjs.com/package/@primer/primitives?activeTab=code
        -- find proper `ansi` key

        -- dark
        -- background = '#0a0a0a',
        -- foreground = '#cecfcf'
        -- ansi = {'#24292f', '#ff8182', '#4ac26b', "#d4a72c", "#54aeff", "#c297ff", "#76e3ea", '#d0d7de'},
        -- brights = {'#32383f', '#ffaba8', '#6fdd8b', "#eac54f", "#80ccff", "#d8b9ff", "#b3f0ff", '#d0d7de'}

        -- light
        -- background = '#fafbf9',
        -- foreground = '#3a3a3a',
        -- ansi = {"#24292f", "#cf222e", "#116329", "#4d2d00", "#0969da", "#8250df", "#1b7c83", "#6e7781"},
        -- brights = {"#57606a", "#a40e26", "#1a7f37", "#633c01", "#218bff", "#a475f9", "#3192aa", "#8c959f"}
    },

    font = wezterm.font 'JetBrains Mono',
    window_background_opacity = 0.64,
    --    enable_tab_bar = false,
    hide_tab_bar_if_only_one_tab = true
}
