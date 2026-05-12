# makepkg configuration (Arch Linux)

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".makepkg.conf",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
