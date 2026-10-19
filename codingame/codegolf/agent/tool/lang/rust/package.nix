{ pkgs, ... }:
pkgs.symlinkJoin {
  name = "cg-rust-1.90.0";
  paths = [
    pkgs.rust-bin.stable."1.90.0".minimal
    pkgs.gcc
    pkgs.util-linux
  ];
}
