# Waybar status bar configuration
# https://github.com/Alexays/Waybar

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "waybar",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
