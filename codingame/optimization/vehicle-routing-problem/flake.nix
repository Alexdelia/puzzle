{
  description = "vehicle-routing-problem: HGS-CVRP solver";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        hgs = pkgs.stdenv.mkDerivation {
          pname = "hgs-cvrp";
          version = "2.0.0-unstable-2025-03-26";
          src = pkgs.fetchFromGitHub {
            owner = "vidalt";
            repo = "HGS-CVRP";
            rev = "1a927955cd2861a29d978f0d359d6e647db9319c";
            hash = "sha256-quaLrOcAh2dxpHNV5jebMFQbEIpi0GCrjhNth/oz5PA=";
          };
          nativeBuildInputs = [ pkgs.cmake ];
          cmakeFlags = [
            "-DCMAKE_BUILD_TYPE=Release"
            "-DBUILD_TESTING=OFF"
          ];
        };
      in
      {
        packages = {
          inherit hgs;
          default = hgs;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            hgs
          ];
        };
      }
    );
}
