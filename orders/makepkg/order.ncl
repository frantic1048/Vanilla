# makepkg configuration (Arch Linux)

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".makepkg.conf",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
