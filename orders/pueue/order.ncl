# Pueue task manager configuration
# https://github.com/Nukesor/pueue

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        source = "pueue.yml",
        prefix = ["~/Library/Application Support/pueue/"],
        when = { os = ["darwin"] },
      },
      {
        source = "pueue.yml",
        prefix = ["~/.config/pueue/"],
        when = { os = ["linux"] },
      },
    ],
  },
} | Order
