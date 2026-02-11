# WezTerm terminal configuration
# https://wezfurlong.org/wezterm/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/wezterm/"],
    files = [
      {
        source = "wezterm.lua",
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
