# X11 configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".xprofile",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
