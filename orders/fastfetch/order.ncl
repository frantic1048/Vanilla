# Fastfetch configuration
# https://github.com/fastfetch-cli/fastfetch

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/fastfetch/"],
    files = [
      {
        source = "config.jsonc",
        format = "json",
        config = {
          "$schema" = "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",
          logo = {
            padding = {
              top = 2,
            },
          },
          modules = [
            "title",
            "separator",
            "os",
            "host",
            "bootmgr",
            "chassis",
            "kernel",
            "packages",
            "shell",
            "editor",
            {
              type = "display",
              preciseRefreshRate = true,
            },
            "lm",
            "de",
            "wm",
            "icons",
            "terminal",
            "terminalfont",
            {
              type = "cpu",
              showPeCoreCount = true,
              temp = true,
            },
            "memory",
            "swap",
            "disk",
            "break",
            "colors",
          ],
        },
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
