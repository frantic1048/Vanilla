# Fcitx input method configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "fcitx",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
