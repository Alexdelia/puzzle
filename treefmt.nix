_: {
  projectRootFile = ".git/config";

  programs = {
    rustfmt = {
      enable = true;
      edition = "2024";
    };

    alejandra.enable = true;
    deadnix.enable = true;
    statix.enable = true;
  };

  settings.global.excludes = [
    ".gitingore"

    "*.lock"

    ".env*"

    "*.png"
    "*.ico"

    "*.toml"

    "codingame/code_golf/*"
  ];
}
