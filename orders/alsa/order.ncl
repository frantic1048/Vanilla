# ALSA audio configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".asoundrc",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
