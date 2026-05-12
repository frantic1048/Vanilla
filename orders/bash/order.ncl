# Bash shell configuration
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".bashrc",
        immutable = true,
      },
      {
        from_file = ".profile",
        immutable = true,
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
