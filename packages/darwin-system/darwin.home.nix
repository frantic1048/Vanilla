{ pkgs, config, lib, ... }:

{
  home = {
    stateVersion = "22.11";
  };

  programs.home-manager.enable = true;
  imports = [
    ../wezterm/program.home.nix
    ../git/program.home.nix
    ../alacritty/program.home.nix
  ];
}
