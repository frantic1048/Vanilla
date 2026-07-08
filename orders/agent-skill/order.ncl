let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = [
      # Codex and friends
      "~/.agents",

      # Claude Code
      "~/.claude"
    ],

    files = [
      {
        from_file = "skills/local-tooling",
        symlink = true
      },
    ],
  },
} | Order
