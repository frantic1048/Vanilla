# tealdeer (tldr client) configuration
# https://dbrgn.github.io/tealdeer/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        source = "config.toml",
        prefix = ["~/Library/Application Support/tealdeer/"],
        when = { os = ["darwin"] },
        config = {
          updates = {
            auto_update = true,
          },
        },
      },
      {
        source = "config.toml",
        prefix = ["~/.config/tealdeer/"],
        when = { os = ["linux"] },
        config = {
          updates = {
            auto_update = true,
          },
        },
      },
    ],
  },
} | Order
