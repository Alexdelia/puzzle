{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "eabc38219184cc3e04a974fe31857d8e0eac098d";
    sha256 = "04ffwp2gzq0hhz7siskw6qh9ys8ragp7285vi1zh8xjksxn1msc5";
  };
in
pkgs.symlinkJoin {
  name = "cg-c-gcc-11.2.0";
  paths = [
    pkgs.gcc11
    pkgs.util-linux
  ];
}
