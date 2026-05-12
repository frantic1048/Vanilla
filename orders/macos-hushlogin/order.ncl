# Suppress the "Last login" banner in new macOS terminal sessions.

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".hushlogin",
        immutable = true,
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
