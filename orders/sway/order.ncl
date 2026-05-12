# Sway Wayland compositor configuration
# https://swaywm.org/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/sway/"],
    files = [
      {
        from_file = "config",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
