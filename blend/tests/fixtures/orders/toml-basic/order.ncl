{
  blend = {
    prefix = ["~/.config/toml-basic/"],
    files = [
      {
        name = "config.toml",
        from_config = {
          key = "value",
          number = 42,
          nested = {
            inner = true,
          },
        },
      },
    ],
  },
}
