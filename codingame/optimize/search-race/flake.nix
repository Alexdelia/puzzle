{
  description = "search-race dev shell (cudarc + CUDA toolkit)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        cudaPkgs = pkgs.cudaPackages_12;
        cuda = pkgs.symlinkJoin {
          name = "cuda";
          paths = with cudaPkgs; [
            cuda_nvcc
            cuda_nvrtc
            cuda_nvrtc.lib
            cuda_cudart
            cuda_cccl
          ];
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt

            cuda

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

          shellHook = ''
            export CUDA_HOME="${cuda}"
            export CUDA_PATH="${cuda}"

            if [ -d /run/opengl-driver/lib ]; then
            	export LD_LIBRARY_PATH="/run/opengl-driver/lib:${cuda}/lib:$LD_LIBRARY_PATH"
            else
            	export LD_LIBRARY_PATH="${cuda}/lib:$LD_LIBRARY_PATH"
            fi
          '';
        };
      }
    );
}
