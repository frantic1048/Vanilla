# Elvish shell configuration
# https://elv.sh/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "elvish",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
