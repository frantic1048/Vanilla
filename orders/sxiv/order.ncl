# sxiv image viewer configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "sxiv",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
