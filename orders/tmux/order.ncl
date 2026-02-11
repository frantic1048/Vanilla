# tmux terminal multiplexer configuration
# https://github.com/tmux/tmux

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".tmux.conf",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
