# Terraform CLI configuration
# https://developer.hashicorp.com/terraform/cli/config/config-file
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/"],
    files = [
      {
        # fixed location
        # https://developer.hashicorp.com/terraform/cli/config/config-file#locations
        name = ".terraformrc",
        from_file = ".terraformrc.hcl",
      },
      # terraform requires plugin_cache_dir to exist but does not create it.
      # let blend own the existence of the dir ¯\_(ツ)_/¯
      {
        name = "plugin-cache",
        from_file = "plugin-cache",
        prefix = ["~/.cache/terraform/"],
        exclude = [".gitkeep"],
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
