{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    devenv,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };

        treefmtEval = inputs.treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
      in {
        packages = {
          devenv-up = self.devShells.${system}.default.config.procfileScript;
          devenv-test = self.devShells.${system}.default.config.test;
        };

        # nix develop
        devShells.default = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            {
              # https://devenv.sh/packages/
              packages =
                (with pkgs; [
                  git

                  ruff
                  ty

                  cargo-flamegraph
                  hyperfine
                  perf
                  libllvm

                  imagemagick
                  bc
                ])
                ++ (with pkgs.python3Packages; [
                  numpy
                  aocd
                  tqdm
                ]);

              # https://devenv.sh/languages/
              languages = {
                rust = {
                  enable = true;
                  channel = "stable";
                  version = "1.90.0"; # https://www.codingame.com/playgrounds/40701/help-center/languages-versions
                };

                python = {
                  enable = true;
                };
              };
            }
          ];
        };

        # nix fmt
        formatter = treefmtEval.config.build.wrapper;
      }
    );
}
