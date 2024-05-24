{ config, pkgs, ... }:
{
  # List packages installed in system profile. To search by name, run:
  # $ nix-env -qaP | grep wget
  environment.systemPackages = with pkgs; [
    # dev
    git
    delta # pager for git, fancy
    git-lfs
    git-annex
    vim # text editor
    nano # text editor
    o # text editor
    nixpkgs-fmt # nix
    cloc # code stats
    scc # code stats, fancy
    tmux
    shellcheck
    go
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
    htop
    ldns # drill
    mtr

    # file system
    fd # find
    exa # ls
    bat # like cat, fancy

    qalculate-gtk
    starship # shell prompt
  ];
  nixpkgs.config.packageOverrides = pkgs: {
    htop = pkgs.htop.overrideAttrs
      (oldAttrs: {
        systemdSupport = false;
        sensorsSupport = false;
      });
  };
  nixpkgs.config.allowUnfree = true;


  # Use a custom configuration.nix location.
  # $ darwin-rebuild switch -I darwin-config=$HOME/.config/nixpkgs/darwin/configuration.nix
  # environment.darwinConfig = "$HOME/.config/nixpkgs/darwin/configuration.nix";

  # Auto upgrade nix package and the daemon service.
  services.nix-daemon.enable = true;
  nix.package = pkgs.nix;
  nix.extraOptions = ''
    experimental-features = nix-command flakes
  '';

  # Create /etc/zshrc that loads the nix-darwin environment.
  programs.zsh.enable = true; # default shell on catalina
  # programs.fish.enable = true;

  # Used for backwards compatibility, please read the changelog before changing.
  # $ darwin-rebuild changelog
  system.stateVersion = 4;
  system.defaults = {
    # defaults read .GlobalPreferences
    ".GlobalPreferences" = {
      # Disable mouse acceleration
      # https://apple.stackexchange.com/questions/439131/how-to-permanently-disable-mouse-acceleration-macos-monterey
      "com.apple.mouse.scaling" = "-1";
    };
    "NSGlobalDomain" = {
      AppleInterfaceStyle = "Dark";
      AppleShowScrollBars = "WhenScrolling";
      NSWindowResizeTime = 0.001;
      AppleICUForce24HourTime = true;
      KeyRepeat = 2;
      "com.apple.keyboard.fnState" = true;
    };
    dock = {
      autohide = true;
    };
  };
}
