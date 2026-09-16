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
        # only handle global tool and versions
        name = ".proto/.prototools",
        format = "toml",
        from_config =
          let global_version = fun { target } =>
            if target == 'Absent then "latest" else target
          in
          {
            bun = global_version,
            deno = global_version,
            go = global_version,
            proto = global_version,
            python = global_version,
            rust = global_version,
            uv = global_version,
            npm = global_version,
            node = global_version,
            pnpm = global_version,
            yarn = global_version,
          }
          # trim any non-global tool version fields nor undesired global tools
          # non tool version settings should go ~/.prototools
          |> blend.with_target_only 'Absent
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
        name = ".proto/tools/node/globals/bin/.gitkeep",
        from_config = m%"
          We need to manually create this folder to workaround the following issue:
          https://github.com/moonrepo/proto/issues/669
        "%,
      }
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
