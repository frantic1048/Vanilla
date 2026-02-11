# Rofi application launcher configuration
# https://github.com/davatorium/rofi

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/rofi/"],
    files = [
      {
        source = "config.rasi",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
