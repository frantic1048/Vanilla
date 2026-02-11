# User scripts

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = "bin",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
