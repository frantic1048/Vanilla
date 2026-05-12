# tmux terminal multiplexer configuration
# https://github.com/tmux/tmux
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/tmux/"],
    files = [
      {
        from_file = "tmux.conf",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
