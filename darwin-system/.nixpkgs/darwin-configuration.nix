{ config, pkgs, ... }:

{
  # List packages installed in system profile. To search by name, run:
  # $ nix-env -qaP | grep wget
  environment.systemPackages = with pkgs; [
    # dev
    git
    git-lfs
    vim # text editor
    nano # text editor
    nixpkgs-fmt # nix
    cloc # code stats
    scc # code stats, fancy
    tmux
    # pinentry_mac, useful ?

    # scripting
    ripgrep # regex
    jq # JSON
    unixtools.watch

    # system
    stow # dotfiles
    inxi # system info
    neofetch # system info, fancy
    mas # Mac App Store CLI
    coreutils

    #htop # FIXME: segfault

    # file system
    fd # find
    exa # ls
    bat # like cat, fancy

    qalculate-gtk
    starship # shell prompt
  ];


  # Use a custom configuration.nix location.
  # $ darwin-rebuild switch -I darwin-config=$HOME/.config/nixpkgs/darwin/configuration.nix
  # environment.darwinConfig = "$HOME/.config/nixpkgs/darwin/configuration.nix";

  # Auto upgrade nix package and the daemon service.
  services.nix-daemon.enable = true;
  nix.package = pkgs.nix;

  # Create /etc/zshrc that loads the nix-darwin environment.
  programs.zsh.enable = true; # default shell on catalina
  # programs.fish.enable = true;

  # Used for backwards compatibility, please read the changelog before changing.
  # $ darwin-rebuild changelog
  system.stateVersion = 4;
}
