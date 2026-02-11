# Swayshot screenshot tool configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "swayshot.sh",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
