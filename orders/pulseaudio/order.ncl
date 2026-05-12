# PulseAudio configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "pulse",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
