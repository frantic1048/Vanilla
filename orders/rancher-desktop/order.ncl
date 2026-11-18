let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/Library/Application Support/rancher-desktop/"],
    when = { os = ["darwin"] },
    files = [
      {
        name = "lima/_config/override.yaml",
        from_config = {
          disk = "30GiB",
          # reverse-sshfs stopped resolving symlinks (composer's path-repository
          # links in the php container); virtiofs is the native share for vz VMs.
          mountType = "virtiofs",
        }
      }
    ],
  },
} | Order
