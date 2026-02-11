# Sapling SCM configuration
# https://sapling-scm.com/

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        source = "sapling.conf",
        prefix = ["~/Library/Preferences/sapling/"],
        when = { os = ["darwin"] },
      },
      {
        source = "sapling.conf",
        prefix = ["~/.config/sapling/"],
        when = { os = ["linux"] },
      },
    ],
  },
} | Order
