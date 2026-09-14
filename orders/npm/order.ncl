# npm configuration
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        name = ".npmrc",
        format = "equals_record_lines",
        from_config = {
          prefix = "~/.local/share/npm",
          "save-exact" = "true",
          "save-prefix" = "",

          "proxy" = 'Unmanaged,
          "https-proxy" = 'Unmanaged,
          "registry" = 'Unmanaged,
          "_authToken" = 'Unmanaged,
        },
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
