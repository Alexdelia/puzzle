{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "50ab793786d9de88ee30ec4e4c24fb4236fc2674";
    sha256 = "1s2gr5rcyqvpr58vxdcb095mdhblij9bfzaximrva2243aal3dgx";
  };
  inherit (pkgs.ocaml-ng.ocamlPackages_4_14) ocaml;
in
pkgs.symlinkJoin {
  name = "cg-ocaml-${ocaml.version}";
  paths = [
    ocaml
    pkgs.gcc
  ];
}
