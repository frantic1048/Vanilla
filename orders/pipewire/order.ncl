# PipeWire audio configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        from_file = "pipewire",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
