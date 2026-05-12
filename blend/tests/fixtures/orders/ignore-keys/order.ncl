{
  blend = {
    prefix = ["~/.config/ignore-keys/"],
    ignore = ["timestamp"],
    files = [
      {
        name = "config.toml",
        from_config = {
          theme = "dark",
          font_size = 14,
        },
      },
    ],
  },
}
