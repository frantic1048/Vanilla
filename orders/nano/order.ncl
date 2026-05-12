# nano text editor configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/nano/"],
    files = [
      {
        from_file = "nanorc",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
