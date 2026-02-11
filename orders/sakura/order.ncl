# Sakura terminal configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/sakura/"],
    files = [
      {
        source = "sakura.conf",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
