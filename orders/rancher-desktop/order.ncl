let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/Library/Application Support/rancher-desktop/"],
    when = { os = ["darwin"] },
    files = [
      {
        name = "lima/_config/override.yaml",
        from_config = {
          disk = "30GiB"
        }
      }
    ],
  },
} | Order
