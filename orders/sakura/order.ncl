# Sakura terminal configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/sakura/"],
    files = [
      {
        from_file = "sakura.conf",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
