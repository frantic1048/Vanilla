let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    files = [
      {
        from_file = "skills/local-tooling",
        symlink = true
      },
    ],

    prefix = ["~/.codex"],
  },
} | Order
