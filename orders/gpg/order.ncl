# GnuPG configuration
let metadata = import "../metadata.ncl" in
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        from_file = ".gnupg",
      },
      {
        # gpg-agent also serves SSH (SSH_AUTH_SOCK), so pinentry must work in
        # GUI contexts. Linux keeps the distribution default pinentry.
        name = ".gnupg/gpg-agent.conf",
        format = "space_record_lines",
        from_config =
          {
            pinentry-program =
              metadata.arch
              |> match {
                "aarch64" => "/opt/homebrew/bin/pinentry-mac",
                _ => "/usr/local/bin/pinentry-mac",
              },
          }
          # other agent options (cache TTLs, etc.) stay host-local
          |> blend.with_target_only 'Unmanaged,
        when = { os = ["darwin"] },
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
