# Display color profiles (ICC)

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.local/share/"],
    files = [
      {
        from_file = "DisplayCAL",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
