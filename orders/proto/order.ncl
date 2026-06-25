# Proto toolchain manager configuration
# https://moonrepo.dev/proto
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~"],
    files = [
      {
        from_file = ".proto/.prototools",
      },
      {
        from_file = ".proto/tools/node/globals/bin/.gitkeep",
      }
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
