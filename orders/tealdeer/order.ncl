# tealdeer (tldr client) configuration
# https://dbrgn.github.io/tealdeer/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        name = "config.toml",
        prefix = ["~/Library/Application Support/tealdeer/"],
        when = { os = ["darwin"] },
        from_config = {
          updates = {
            auto_update = true,
          },
        },
      },
      {
        name = "config.toml",
        prefix = ["~/.config/tealdeer/"],
        when = { os = ["linux"] },
        from_config = {
          updates = {
            auto_update = true,
          },
        },
      },
    ],
  },
} | Order
