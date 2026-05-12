# SketchyBar configuration
# https://felixkratz.github.io/SketchyBar/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "sketchybar",
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
