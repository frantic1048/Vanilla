# Proto toolchain manager configuration
# https://moonrepo.dev/proto

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".proto",
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
