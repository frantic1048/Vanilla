# Mako notification daemon configuration
# https://github.com/emersion/mako

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/mako/"],
    files = [
      {
        source = "config",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
