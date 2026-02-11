# Picom compositor configuration
# https://github.com/yshui/picom

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/picom/"],
    files = [
      {
        source = "picom.conf",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
