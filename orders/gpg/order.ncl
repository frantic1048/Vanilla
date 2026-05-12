# GnuPG configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".gnupg",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
