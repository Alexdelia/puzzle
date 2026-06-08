{
  description = "search-race dev shell (cuda-oxide toolchain)";

  inputs = {
    cuda-oxide.url = "github:NVlabs/cuda-oxide";
    nixpkgs.follows = "cuda-oxide/nixpkgs";
    flake-utils.follows = "cuda-oxide/flake-utils";
  };

  outputs = {
    cuda-oxide,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.mkShell {
          inputsFrom = [cuda-oxide.devShells.${system}.default];

          packages = with pkgs; [
            python3

            ruff
            ty

            cargo-flamegraph
            hyperfine
            perf
            libllvm

            imagemagick
            bc
          ];
        };
      }
    );
}
