# Git configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/git/"],
    files = [
      {
        source = "git",
        prefix = ["~/.config/"],
      },
      {
        source = "gitk",
        ignore = [
          "^set geometry",
        ],
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
