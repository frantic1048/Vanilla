# Neofetch configuration (deprecated)

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/neofetch/"],
    files = [
      {
        source = "config.conf",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
