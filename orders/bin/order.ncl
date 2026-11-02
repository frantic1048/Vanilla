# User scripts
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.local/bin/"],
    files = [
      {
        kind = "symlink",
        name = "van",
        from_file = "bin",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
