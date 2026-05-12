# Git configuration
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        from_file = "git",
        prefix = ["~/.config/"],
      },
      {
        from_file = "gitk",
        prefix = ["~/.config/git/"],
        ignore = [
          "^set geometry",
        ],
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
