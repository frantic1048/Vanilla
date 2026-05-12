# npm configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        name = ".npmrc",
        format = "equals_record_lines",
        ignore = [
          "proxy",
          "https-proxy",
          "registry",
          "_authToken",
        ],
        from_config = {
          prefix = "~/.local/share/npm",
          "save-exact" = "true",
          "save-prefix" = "",
        },
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
