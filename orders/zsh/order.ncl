# Zsh shell configuration
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".zshrc",
        immutable = true,
      },
      {
        from_file = ".zshenv",
        immutable = true,
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
