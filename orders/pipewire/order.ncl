# PipeWire audio configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "pipewire",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
