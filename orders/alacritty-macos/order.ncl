# Alacritty terminal configuration (macOS)
# https://alacritty.org/config-alacritty.html

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/alacritty/"],
    files = [
      {
        source = "alacritty.toml",
        config = {
          window = {
            decorations = "Buttonless",
            opacity = 0.7,
          },

          font = {
            size = 12.0,

            bold = {
              family = "JetBrains Mono",
              style = "Bold",
            },

            bold_italic = {
              family = "Jetbrains Mono",
              style = "Bold Italic",
            },

            italic = {
              family = "Jetbrains Mono",
              style = "Italic",
            },

            normal = {
              family = "Jetbrains Mono",
              style = "Regular",
            },
          },

          # How to easily clear all bindings ?
          bindings = {},
        },
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
