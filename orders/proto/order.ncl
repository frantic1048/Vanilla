# Proto toolchain manager configuration
# https://moonrepo.dev/proto
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~"],
    files = [
      {
        # https://moonrepo.dev/docs/proto/config#locations
        # proto global configuration file, the only proto config Vanilla
        # manages. It applies from any directory, while ~/.prototools is only
        # found by upwards traversal from paths below ~.
        # Global tool versions belong to each host (`proto install --pin
        # global`, system-maintenance.nu) and stay unmanaged.
        name = ".proto/.prototools",
        format = "toml",
        from_config =
          {
            # Plugins must live here: proto only loads plugins from the global
            # config or from trusted configs. Relative locators resolve from
            # ~/.proto/.
            # `proto install` defaults to upwards config mode, which ignores
            # these plugins; pass `-c upwards-global` when installing them.
            plugins = {
              tools = {
                blend = 'Enforce "file://./vanilla-plugins/blend.toml",
              },
            },
            tools = {
              node = { "bundled-npm" = true },
              npm = { "shared-globals-dir" = true },
            },
            settings = {
              "auto-install" = true,
              telemetry = false,
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
        # We need to manually create this folder to workaround the following issue:
        # https://github.com/moonrepo/proto/issues/669
        kind = "directory",
        name = ".proto/tools/node/globals/bin",
      }
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
