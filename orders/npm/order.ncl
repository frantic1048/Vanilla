# npm configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".npmrc",
        format = "equal_delimited_record",
        ignore = [
          "proxy",
          "https-proxy",
          "registry",
          "_authToken",
        ],
        config = {
          prefix = "~/.local/share/npm",
          "save-exact" = "true",
          "save-prefix" = "",
        },
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
