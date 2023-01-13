{
  programs.git.enable = false;

  # user
  programs.git.userName = "frantic1048";
  programs.git.userEmail = "i@frantic1048.com";
  programs.git.signing.key = "22D8A46B2CDA6605A1C0CFD1E060B3E215CE49BB";

  # diff pager
  programs.git.delta.enable = true;

  # ignore
  programs.git.ignores = [
    ".DS_Store" # macOS
    ".directory" # Dolphin
    "node_modules" # Node.js
    "*.log" # Logs
  ];

  # how to nix-sops ?
  programs.git.includes = [
    {
      path = "~/.gitconfig.user.work";
      condition = "gitdir:~/work/";
    }
  ];

  # filters
  programs.git.lfs.enable = true;

  # others
  programs.git.extraConfig = {
    core.excludesfile = "~/.gitignore";
    diff.algorithm = "histogram";
    merge.ff = "only";
    push.default = "simple";
    pull.ff = "only";
    rebase.autoSquash = true;
    rebase.instructionFormat = "%aN\t%s";
    http.postBuffer = 1048576000;
    # filters looks too hard...
    # https://github.com/nix-community/home-manager/issues/542#issuecomment-457954666
  };
}
