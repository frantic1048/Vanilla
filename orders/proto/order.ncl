# Proto toolchain manager configuration
# https://moonrepo.dev/proto
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~"],
    files = [
      {
        # https://moonrepo.dev/docs/proto/config#locations
        # proto global configuration file
        # Only registers custom plugins. Global tool versions belong to each
        # host (`proto install --pin global`, system-maintenance.nu).
        name = ".proto/.prototools",
        format = "toml",
        from_config =
          {
            # Plugins must live here: proto only loads plugins from the global
            # config or from trusted configs, and ~/.prototools is not trusted.
            # Relative locators resolve from ~/.proto/.
            plugins = {
              tools = {
                blend = 'Enforce "file://./vanilla-plugins/blend.toml",
              },
            },
          }
          |> blend.with_target_only 'Unmanaged
      },
      {
        # Custom proto plugins registered by the global config above
        from_file = "blend.toml",
        name = ".proto/vanilla-plugins/blend.toml",
      },
      {
        # proto user configuration file
        name = ".prototools",
        format = "toml",
        from_config = {
          tools = {
            node = { "bundled-npm" = true },
            npm = { "shared-globals-dir" = true },
          },
          settings = {
            "auto-install" = true,
            telemetry = false,
          },
        }
      },
      {
        # We need to manually create this folder to workaround the following issue:
        # https://github.com/moonrepo/proto/issues/669
        kind = "directory",
        name = ".proto/tools/node/globals/bin",
      }
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
