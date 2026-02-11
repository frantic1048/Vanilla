# Neovim configuration (plaintext)
# Source files are copied as-is to target

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "nvim",  # Copy orders/neovim/nvim/ to target
        # No config section - source directory copied as-is
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
