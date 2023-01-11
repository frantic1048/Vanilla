macOS configuration, managed with `nix-darwin`: https://github.com/LnL7/nix-darwin/

# Bootstrap

Install Nix: https://nixos.org/download.html#nix-install-macos

Setup Nix, `nix-darwin` and `home-manager`:

```
nix-channel --add https://nixos.org/channels/nixpkgs-unstable
nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
nix-build https://github.com/LnL7/nix-darwin/archive/master.tar.gz -A installer
```

TODO: add more explanation.

Apply configuration to system: `darwin-rebuild switch`

# Memo

Useful commands:

- List homebrew packages: `brew leaves --installed-on-request`
- Search nix packages: `nix-env -qaP | grep wget`
