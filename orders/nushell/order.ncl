# Nushell configuration
# https://www.nushell.sh/
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        from_file = "nushell",
        prefix = ["~/Library/Application Support/"],
        exclude = ["nushell/history.txt"],
        when = { os = ["darwin"] },
      },
      {
        from_file = "nushell",
        prefix = ["~/.config/"],
        exclude = ["nushell/history.txt"],
        when = { os = ["linux"] },
      },
    ],
  },
} | Order
