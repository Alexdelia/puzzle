{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "d8df4205639de026f1831f3712d52cde89d4892a";
    sha256 = "0awq2mi329qck2izc5sknhxr1ykk45w3fmc9b1zis2ydvfs70hjk";
  };
in
pkgs.go_1_18
