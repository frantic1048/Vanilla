# yabai tiling window manager configuration
# https://github.com/koekeishiya/yabai

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/yabai/"],
    files = [
      {
        from_file = "yabairc",
      },
      {
        from_file = "yabairc.yabai",
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
