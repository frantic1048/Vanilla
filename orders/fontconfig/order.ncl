# Fontconfig configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "fontconfig",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
