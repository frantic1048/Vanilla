# Nushell configuration
# https://www.nushell.sh/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        source = "nushell",
        prefix = ["~/Library/Application Support/"],
        when = { os = ["darwin"] },
      },
      {
        source = "nushell",
        prefix = ["~/.config/"],
        when = { os = ["linux"] },
      },
    ],
  },
} | Order
