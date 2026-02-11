# PulseAudio configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "pulse",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
