# Display color profiles (ICC)

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.local/share/"],
    files = [
      {
        source = "DisplayCAL",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
