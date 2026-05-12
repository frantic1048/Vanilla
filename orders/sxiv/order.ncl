# sxiv image viewer configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "sxiv",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
