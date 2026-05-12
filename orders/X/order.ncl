# X11 configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".xprofile",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
