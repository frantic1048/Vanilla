macOS configuration, managed with `nix-darwin`: https://github.com/LnL7/nix-darwin/

# Bootstrap

1. Install Nix: https://nixos.org/download.html#nix-install-macos
2. Install nix-darwin: https://github.com/LnL7/nix-darwin/#install
3. TODO: add more explanation
4. apply configuration to system: `darwin-rebuild switch`

# Memo

Useful commands:

- List homebrew packages: `brew leaves --installed-on-request`
- Search nix packages: `nix-env -qaP | grep wget`
