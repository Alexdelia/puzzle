{ pkgs, ... }:
let
  version = "2.099.0";
  dmd = pkgs.stdenv.mkDerivation {
    pname = "dmd-bin";
    inherit version;
    src = pkgs.fetchurl {
      url = "https://downloads.dlang.org/releases/2022/dmd.${version}.linux.tar.xz";
      sha256 = "19mm0cyk7sdi7dxni4901hyzlwk2d99j8gwd2pzaphxv2vnzrh6l";
    };
    nativeBuildInputs = [
      pkgs.autoPatchelfHook
      pkgs.makeWrapper
    ];
    buildInputs = [ pkgs.stdenv.cc.cc.lib ];
    installPhase = ''
      mkdir -p $out/share/dmd2/linux $out/bin
      cp -r linux/bin64 linux/lib64 $out/share/dmd2/linux/
      cp -r src $out/share/dmd2/
      for tool in dmd rdmd; do
        makeWrapper $out/share/dmd2/linux/bin64/$tool $out/bin/$tool \
          --set CC ${pkgs.stdenv.cc}/bin/cc \
          --prefix PATH : ${pkgs.stdenv.cc}/bin
      done
    '';
  };
in
dmd
