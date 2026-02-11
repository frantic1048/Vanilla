# SketchyBar configuration
# https://felixkratz.github.io/SketchyBar/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "sketchybar",
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
