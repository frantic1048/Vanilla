{
  blend = {
    prefix = ["~/.config/literal-dotted-keys/"],
    files = [
      {
        name = "settings.json",
        from_config = {
          "[javascript]" = {
            "editor.codeActionsOnSave" = { "source.fixAll.eslint" = "explicit" },
            "editor.defaultFormatter" = "esbenp.prettier-vscode",
          },
        },
      },
    ],
  },
}
