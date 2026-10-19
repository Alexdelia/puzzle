{ inputs, system, ... }:
inputs.nixpkgs-python.packages.${system}."3.11.5"
