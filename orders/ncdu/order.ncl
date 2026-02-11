# ncdu disk usage analyzer configuration
# https://dev.yorhel.nl/ncdu

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/ncdu/"],
    files = [
      {
        source = "config",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
