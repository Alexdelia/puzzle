{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "6233296137da293de6719c7b0cf9435a67dadc8c";
    sha256 = "1kll3bs7f566m059d52x4z7za1yy17gv09l800zw4fdlxryslglg";
  };
in
pkgs.nodejs_20
