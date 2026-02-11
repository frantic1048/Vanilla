# tint2 panel configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/tint2/"],
    files = [
      {
        source = "tint2rc",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
