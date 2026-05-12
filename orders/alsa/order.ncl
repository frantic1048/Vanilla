# ALSA audio configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".asoundrc",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
