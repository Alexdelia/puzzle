{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "22.11";
    sha256 = "11w3wn2yjhaa5pv20gbfbirvjq6i3m7pqrq2msf0g7cv44vijwgw";
  };
in
pkgs.lua5_4
