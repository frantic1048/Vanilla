# mise configuration
# https://mise.jdx.dev/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/mise/"],
    files = [
      {
        source = "config.toml",
        config = {
          settings = {
            # Disable most of the core plugins
            # Only Ruby is supposed to be enabled
            # https://mise.jdx.dev/lang/bun.html
            disable_tools = [
              "node",
              "bun",
              "deno",
              "python",
              "rust",
              "java",
              "go",
              "erlang",
            ],
          },
        },
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
