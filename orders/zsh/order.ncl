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
      {
        from_file = ".zprofile",
        immutable = true,
      },
      {
        name = ".zlogin",
        immutable = true,
        from_config = "",
      }
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
