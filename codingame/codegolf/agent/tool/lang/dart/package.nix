{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "ee09932cedcef15aaf476f9343d1dea2cb77e261";
    sha256 = "1xz5pa6la2fyj5b1cfigmg3nmml11fyf9ah0rnr4zfgmnwimn2gn";
  };
in
pkgs.dart
