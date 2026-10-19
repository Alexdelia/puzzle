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
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      inherit (pkgs) lib;

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

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          refereeRust
          jq
          curl
          util-linux
          time
          coreutils
        ];

        env = lib.mapAttrs' (name: package: lib.nameValuePair (envName name) "${package}") langPackages;
      };
    };
}
