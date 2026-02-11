# Bash shell configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".bashrc",
      },
      {
        source = ".profile",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
