{ fetchNixpkgs, ... }:
let
  nixos1809 = fetchNixpkgs {
    rev = "a7e559a5504572008567383c3dc8e142fa7a8633";
    sha256 = "16j95q58kkc69lfgpjkj76gw5sx8rcxwi3civm0mlfaxxyw9gzp6";
  };
in
nixos1809.haskell.packages.ghc843.ghcWithPackages (
  p: with p; [
    async
    attoparsec
    case-insensitive
    fgl
    hashable
    HUnit
    mtl
    network
    parallel
    parsec
    primitive
    QuickCheck
    random
    regex-base
    regex-compat
    regex-posix
    scientific
    split
    stm
    syb
    text
    unordered-containers
    vector
    zlib
  ]
)
