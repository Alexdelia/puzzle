{ pkgs, ... }:
let
  linker = pkgs.writeShellScript "swift-clang-linker" ''
    args=()
    for arg in "$@"; do
      [[ $arg == -fuse-ld=gold ]] || args+=("$arg")
    done
    exec ${pkgs.clang}/bin/clang "''${args[@]}"
  '';
  debianSonames = pkgs.runCommand "swift-debian-sonames" { } ''
    mkdir -p $out/lib
    ln -s ${pkgs.libedit}/lib/libedit.so.0 $out/lib/libedit.so.2
    ln -s ${pkgs.lib.getLib pkgs.z3}/lib/libz3.so $out/lib/libz3.so.4
    ln -s ${pkgs.ncurses}/lib/libncursesw.so.6 $out/lib/libtinfo.so.6
    ln -s ${pkgs.lib.getLib pkgs.libxml2}/lib/libxml2.so $out/lib/libxml2.so.2
  '';
in
pkgs.stdenv.mkDerivation {
  pname = "cg-swift";
  version = "5.3.3";
  src = pkgs.fetchurl {
    url = "https://download.swift.org/swift-5.3.3-release/ubuntu2004/swift-5.3.3-RELEASE/swift-5.3.3-RELEASE-ubuntu20.04.tar.gz";
    hash = "sha256-4mJPK1bNYwEaoaGF6j+prt8Vfv6G5rIbPqzFaalI514=";
  };
  nativeBuildInputs = [
    pkgs.autoPatchelfHook
    pkgs.xorg.lndir
  ];
  buildInputs = [
    pkgs.stdenv.cc.cc.lib
    pkgs.zlib
    pkgs.ncurses
    debianSonames
    pkgs.libxml2
    pkgs.curl
    pkgs.sqlite
    pkgs.libuuid
  ];
  autoPatchelfIgnoreMissingDeps = true;
  dontStrip = true;
  installPhase = ''
    mkdir -p $out/sysinclude
    cp -a usr/* $out/
    lndir -silent ${pkgs.glibc.dev}/include $out/sysinclude
    lndir -silent ${pkgs.linuxHeaders}/include $out/sysinclude
    substituteInPlace $out/lib/swift/linux/x86_64/glibc.modulemap \
      --replace-quiet /usr/include/x86_64-linux-gnu $out/sysinclude \
      --replace-quiet /usr/include $out/sysinclude
    rm -f $out/bin/clang $out/bin/clang++
    ln -s ${linker} $out/bin/clang
  '';
}
