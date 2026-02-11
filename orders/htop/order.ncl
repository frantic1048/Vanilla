# htop process viewer configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/htop/"],
    files = [
      {
        source = "htoprc",
        ignore = [
          "^tree_view",
          "^color_scheme",
          "^sort_key",
        ],
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
