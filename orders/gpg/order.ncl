# GnuPG configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".gnupg",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
