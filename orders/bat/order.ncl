# bat configuration
# https://github.com/sharkdp/bat

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/bat/"],
    files = [
      {
        source = "config",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
