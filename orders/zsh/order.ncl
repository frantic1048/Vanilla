# Zsh shell configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".zshrc",
      },
      {
        source = ".zshenv",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
