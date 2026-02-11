# Fcitx input method configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "fcitx",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
