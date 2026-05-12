# Neofetch configuration (deprecated)

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/neofetch/"],
    files = [
      {
        from_file = "config.conf",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
