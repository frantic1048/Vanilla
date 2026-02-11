# i3 window manager configuration
# https://i3wm.org/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "i3",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
