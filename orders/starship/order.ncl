# Starship prompt configuration
# https://starship.rs/config/
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        name = "starship.toml",
        from_config = {
          "$schema" = "https://starship.rs/config-schema.json",
          command_timeout = 10000,

          git_branch = {
            style = "bold bright-green",
          },

          shell = {
            disabled = false,
            elvish_indicator = "~",
            format = "[$indicator]($style)",
          },

          # Nerd Fonts
          # https://www.nerdfonts.com/cheat-sheet
          bun = { symbol = "\u{e76f} " },
          c = { symbol = "\u{e771} " },
          cmake = { symbol = "\u{e794} " },
          docker_context = { symbol = "\u{e7b0} " },
          deno = {
            symbol = "\u{e7c0} ",
            detect_files = [
              "deno.json",
              "deno.jsonc",
              "deno.lock",
              "mod.ts",
              "mod.js",
              "deps.ts",
              "deps.js",
              ".dvmrc",
            ],
          },
          dotnet = { symbol = "\u{e77f} " },
          elm = { symbol = "\u{e7ce} " },
          golang = { symbol = "\u{e724} " },
          guix_shell = { symbol = "\u{f325} " },
          lua = { symbol = "\u{e826} " },
          nim = { symbol = "\u{e841} " },
          nix_shell = { symbol = "\u{e843} " },
          nodejs = { symbol = "\u{e719} " },
          ocaml = { symbol = "\u{e84e} " },
          python = { symbol = "\u{e73c} " },
          ruby = { symbol = "\u{e739} " },
          rust = { symbol = "\u{e7a8} " },
          terraform = { symbol = "\u{e8bd} " },
          typst = { symbol = "\u{f37f} " },
        },
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
