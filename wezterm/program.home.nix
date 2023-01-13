let
  extraConfig = builtins.readFile ./.wezterm.lua;
in
{
  programs.wezterm = {
    enable = true;
    inherit extraConfig;
  };
}
