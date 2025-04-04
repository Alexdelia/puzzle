_: {
  projectRootFile = ".git/config";

  programs = {
    rustfmt.enable = true;

    alejandra.enable = true;
    deadnix.enable = true;
    statix.enable = true;
  };

  settings = {
    global.excludes = [
      ".gitingore"

      "*.lock"

      ".env*"

      "*.png"
      "*.ico"

      "*.toml"

      "codingame/code_golf/*"
    ];

    formatter.rustfmt.options = [
      "--config-path"
      (builtins.toString ./.rustfmt.toml)
    ];
  };
}
