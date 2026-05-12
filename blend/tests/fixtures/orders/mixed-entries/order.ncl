{
  blend = {
    prefix = ["~/.config/mixed-entries/"],
    files = [
      {
        name = "config.toml",
        from_config = {
          theme = "dark",
          font_size = 14,
        },
      },
      {
        name = "extra.txt",
        from_file = "extra.txt",
      },
    ],
  },
}
