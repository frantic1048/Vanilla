# Fontconfig configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "fontconfig",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
