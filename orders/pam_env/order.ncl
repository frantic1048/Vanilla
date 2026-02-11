# PAM environment configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        source = ".pam_environment",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
