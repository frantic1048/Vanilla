# Elvish shell configuration
# https://elv.sh/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "elvish",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
