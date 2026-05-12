# Waybar status bar configuration
# https://github.com/Alexays/Waybar

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "waybar",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
