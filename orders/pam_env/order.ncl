# PAM environment configuration

let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".pam_environment",
      },
    ],
    when = { os = ["linux"] },
  },
} | Order
