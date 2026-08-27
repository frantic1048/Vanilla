{
  blend = {
    prefix = ["~/.config/yaml-format/"],
    files = [
      {
        name = "settings.yaml",
        from_config = {
          enabled = true,
          message = "hello",
          nested = {
            count = 2,
          },
          ports = [8080, 8443],
        },
      },
    ],
  },
}
