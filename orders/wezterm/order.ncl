# WezTerm terminal configuration
# https://wezfurlong.org/wezterm/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/wezterm/"],
    files = [
      {
        from_file = "wezterm.lua",
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
