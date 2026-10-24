{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    nixpkgs-python = {
      url = "github:cachix/nixpkgs-python";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }@inputs:
    let
      system = "x86_64-linux";
      inherit (nixpkgs) lib;
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
        config.allowUnfreePredicate = package: lib.getName package == "claude-code";
      };

      fetchNixpkgs =
        { rev, sha256 }:
        import (fetchTarball {
          url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
          inherit sha256;
        }) { inherit system; };

      langDir = ./agent/tool/lang;
      isLang = name: type: type == "directory" && builtins.pathExists (langDir + "/${name}/package.nix");
      langs = builtins.attrNames (lib.filterAttrs isLang (builtins.readDir langDir));
      langPackages = lib.genAttrs langs (
        name:
        import (langDir + "/${name}/package.nix") {
          inherit
            pkgs
            inputs
            system
            fetchNixpkgs
            ;
        }
      );
      envName = name: "CG_" + lib.toUpper (builtins.replaceStrings [ "-" ] [ "_" ] name);

      dash =
        (fetchNixpkgs {
          rev = "eabc38219184cc3e04a974fe31857d8e0eac098d";
          sha256 = "04ffwp2gzq0hhz7siskw6qh9ys8ragp7285vi1zh8xjksxn1msc5";
        }).dash;

      agentPython = pkgs.python3.withPackages (
        python: with python; [
          bitarray
          matplotlib
          more-itertools
          networkx
          numba
          numpy
          ortools
          pillow
          pulp
          python-sat
          pyyaml
          regex
          requests
          scipy
          sortedcontainers
          sympy
          tqdm
          z3-solver
        ]
      );

      refereeRust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "clippy"
          "rustfmt"
          "rust-src"
        ];
      };
    in
    {
      packages.${system} = langPackages;

      legacyPackages.${system}.langArgs = {
        inherit
          pkgs
          inputs
          system
          fetchNixpkgs
          ;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          claude-code
          opencode
          refereeRust
          agentPython
          pypy3
          bashInteractive
          coreutils
          util-linux
          procps
          psmisc
          lsof
          strace
          gdb
          time
          hyperfine
          moreutils
          parallel
          findutils
          gnugrep
          gnused
          gawk
          diffutils
          ripgrep
          fd
          tree
          which
          file
          less
          xxd
          bc
          git
          jq
          curl
          socat
          sqlite
          zip
          unzip
          xz
          perl
          nodejs
          z3
          glpk
          minizinc
        ];

        env = lib.mapAttrs' (name: package: lib.nameValuePair (envName name) "${package}") langPackages // {
          CG_DASH = "${dash}";
          CG_BWRAP = "${pkgs.bubblewrap}";
        };
      };
    };
}
