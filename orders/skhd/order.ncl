# skhd hotkey daemon configuration
# https://github.com/koekeishiya/skhd

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/skhd/"],
    files = [
      {
        from_file = "skhdrc",
      },
    ],
    when = { os = ["darwin"] },
  },
} | Order
