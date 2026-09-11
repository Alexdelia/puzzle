{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nixpkgs-python = {
      url = "github:cachix/nixpkgs-python";
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

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      devenv,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        treefmtEval = inputs.treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
      in
      {
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
              packages = with pkgs; [
                git

                ruff
                ty

                cargo-flamegraph
                hyperfine
                perf
                libllvm

                imagemagick
                bc
              ];

              # https://devenv.sh/languages/
              languages = {
                rust = {
                  enable = true;
                  channel = "stable";
                  version = "1.90.0"; # https://www.codingame.com/playgrounds/40701/help-center/languages-versions
                };

                python = {
                  enable = true;
                  version = "3.11.5"; # https://www.codingame.com/playgrounds/40701/help-center/languages-versions

                  manylinux.enable = true;

                  venv = {
                    enable = true;
                    requirements = ''
                      numpy==1.23.2
                      pandas==1.5.0
                      scipy==1.9.2
                      aocd
                      tqdm
                    '';
                  };
                };
              };

              # https://devenv.sh/git-hooks/
              git-hooks.hooks.treefmt = {
                enable = true;
                package = treefmtEval.config.build.wrapper;
                settings.fail-on-change = true;
              };
            }
          ];
        };

        # nix fmt
        formatter = treefmtEval.config.build.wrapper;
      }
    );
}
