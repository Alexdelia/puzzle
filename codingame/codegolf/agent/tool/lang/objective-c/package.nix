{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "22.05";
    sha256 = "0d643wp3l77hv2pmg2fi7vyxn4rwy0iyr8djcw1h5x72315ck9ik";
  };
in
pkgs.symlinkJoin {
  name = "cg-objective-c-clang-13.0.1";
  paths = [
    pkgs.llvmPackages_13.clang
    pkgs.gnustep.base
    pkgs.gnustep.libobjc
    pkgs.gnustep.make
    pkgs.util-linux
  ];
}
