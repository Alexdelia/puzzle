{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "22.05";
    sha256 = "0d643wp3l77hv2pmg2fi7vyxn4rwy0iyr8djcw1h5x72315ck9ik";
  };
in
pkgs.ruby_3_1
