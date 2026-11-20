# Pyonpyon: a small symbol font (Tippy and friends). README.md lists the glyphs
# and code points and explains how to rebuild Pyonpyon.otf.
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        from_file = "Pyonpyon.otf",
        prefix = ["~/Library/Fonts/"],
        when = { os = ["darwin"] },
      },
      {
        from_file = "Pyonpyon.otf",
        prefix = ["~/.local/share/fonts/"],
        when = { os = ["linux"] },
      },
    ],
  },
} | Order
