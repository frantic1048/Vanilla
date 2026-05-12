# Swayshot screenshot tool configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "swayshot.sh",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
