# Kitty terminal configuration
# https://sw.kovidgoyal.net/kitty/conf/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/kitty/"],
    files = [
      {
        source = "kitty.conf",
        format = "space_delimited_pairs",
        config = [
          # Font
          ["font_size", "16.0"],
          ["font_family", "family=\"JetBrainsMono Nerd Font Mono\""],
          ["bold_font", "auto"],
          ["italic_font", "auto"],
          ["bold_italic_font", "auto"],

          # Symbol map for Nerd Fonts
          ["symbol_map", "U+23FB-U+23FE,U+2665,U+26A1,U+2B58,U+E000-U+E00A,U+E0A0-U+E0A3,U+E0B0-U+E0D4,U+E200-U+E2A9,U+E300-U+E3E3,U+E5FA-U+E6AA,U+E700-U+E7C5,U+EA60-U+EBEB,U+F000-U+F2E0,U+F300-U+F32F,U+F400-U+F4A9,U+F500-U+F8FF,U+F0001-U+F1AF0 Symbols Nerd Font Mono"],

          # Cursor
          ["cursor_trail", "1"],

          # Tab bar
          ["tab_bar_style", "powerline"],
          ["tab_powerline_style", "slanted"],

          # Theme
          ["include", "themes/mocha.conf"],

          # Appearance
          ["background_opacity", "0.6"],
          ["background_blur", "2"],
          ["background", "#000000"],
          ["tab_bar_background", "#2c2421"],
          ["hide_window_decorations", "titlebar-only"],

          # Behavior
          ["close_on_child_death", "yes"],
          ["text_fg_override_threshold", "10"],

          # Shortcuts - clear all for zellij, then re-add essentials
          ["clear_all_shortcuts", "yes"],
          # linux
          ["map", "ctrl+shift+c copy_to_clipboard"],
          ["map", "ctrl+shift+v paste_from_clipboard"],
          # macos
          ["map", "cmd+v paste_from_clipboard"],
          ["map", "cmd+d close_window"],
          ["map", "cmd+q quit"],
        ],
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
