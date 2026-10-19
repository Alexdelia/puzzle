{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "d4f247e89f6e10120f911e2e2d2254a050d0f732";
    sha256 = "0h7z2g4ysb6cnavragn20vhqf33vi5a89g6pydavwg5rivdlx40d";
  };
in
pkgs.dotnet-sdk_8
