# yabai tiling window manager configuration
# https://github.com/koekeishiya/yabai

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/yabai/"],
    files = [
      {
        source = "yabairc",
      },
      {
        source = "yabairc.yabai",
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
