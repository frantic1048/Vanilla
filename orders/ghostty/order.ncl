# Ghostty terminal configuration
# https://ghostty.org/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/ghostty/"],
    files = [
      {
        source = "config",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
