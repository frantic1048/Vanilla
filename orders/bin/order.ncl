# User scripts

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.local/bin/"],
    files = [
      {
        name = "van",
        from_file = "../../bin",
        symlink = true,
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
