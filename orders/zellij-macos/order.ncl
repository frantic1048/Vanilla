# Zellij terminal multiplexer configuration (macOS)
# https://zellij.dev/documentation/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/zellij/"],
    files = [
      {
        source = "config.kdl",
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
