{
  inputs,
  system,
  fetchNixpkgs,
  ...
}:
let
  pkgs = fetchNixpkgs {
    rev = "22.05";
    sha256 = "0d643wp3l77hv2pmg2fi7vyxn4rwy0iyr8djcw1h5x72315ck9ik";
  };
  locales = pkgs.glibcLocales.override {
    allLocales = false;
    locales = [ "en_US.UTF-8/UTF-8" ];
  };
in
pkgs.symlinkJoin {
  name = "cg-bash-5.1.16";
  paths = [
    pkgs.bashInteractive
    pkgs.coreutils
    pkgs.gnused
    pkgs.gnugrep
    pkgs.mawk
    pkgs.bc
    pkgs.xxd
    pkgs.perl
    pkgs.util-linux
    pkgs.findutils
    pkgs.diffutils
    pkgs.gzip
    inputs.nixpkgs-python.packages.${system}."3.11.5"
    locales
  ];
  postBuild = ''
    rm $out/bin/dc
    ln -s mawk $out/bin/awk
  '';
}
