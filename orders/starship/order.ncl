# Starship prompt configuration
# https://starship.rs/config/
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/"],
    files = [
      {
        source = "starship.toml",
        config = {
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
          # MEMO: trailing space is required
          bun = { symbol = " " }, # \ue76f
          c = { symbol = " " }, # \ue771
          cmake = { symbol = " " }, # \ue794
          docker_context = { symbol = " " }, # \ue7b0
          deno = {
            symbol = " ", # \ue7c0
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
          dotnet = { symbol = " " }, # \ue77f
          elm = { symbol = " " }, # \ue7ce
          golang = { symbol = " " }, # \ue724
          guix_shell = { symbol = " " }, # \uf325
          lua = { symbol = " " }, # \ue826
          nim = { symbol = " " }, # \ue841
          nix_shell = { symbol = " " }, # \ue843
          nodejs = { symbol = " " }, # \ue719
          ocaml = { symbol = " " }, # \ue84e
          python = { symbol = " " }, # \ue73c
          ruby = { symbol = " " }, # \ue739
          rust = { symbol = " " }, # \ue7a8
          terraform = { symbol = " " }, # \ue8bd
          typst = { symbol = " " }, # \uf37f
        },
      },
    ],
    when = { os = ["darwin", "linux"] },
  },
} | Order
