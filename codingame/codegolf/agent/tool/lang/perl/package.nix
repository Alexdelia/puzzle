{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "21.11";
    sha256 = "162dywda2dvfj1248afxc45kcrg83appjd0nmdb541hl7rnncf02";
  };
  cpan = pkgs.perl534Packages;
  ArrayUtils = cpan.buildPerlPackage {
    pname = "Array-Utils";
    version = "0.5";
    src = pkgs.fetchurl {
      url = "mirror://cpan/authors/id/Z/ZM/ZMIJ/Array/Array-Utils-0.5.tar.gz";
      sha256 = "0w1pwvnjdpb0n6k07zbknxwx6v7y75p4jxrs594pjhwvrmzippc9";
    };
  };
  Tree = cpan.buildPerlPackage {
    pname = "Tree";
    version = "1.15";
    src = pkgs.fetchurl {
      url = "mirror://cpan/authors/id/R/RS/RSAVAGE/Tree-1.15.tgz";
      sha256 = "1a3ky5v5jkca7ph44md7gyjc5j5vs8m112hcnfx7dnmy1d3sl6yv";
    };
    doCheck = false;
  };
  perl = pkgs.perl534.withPackages (
    p: with p; [
      AlgorithmDiff
      AlgorithmMerge
      ArrayUtils
      CaptureTiny
      ClassXSAccessor
      DateTime
      ListCompare
      ListMoreUtils
      Moose
      PadWalker
      RefUtil
      RegexpAssemble
      SetScalar
      StringUtil
      TextUnidecode
      Tree
      TryTiny
      UnicodeLineBreak
      UnicodeUTF8
      XMLLibXML
      enum
    ]
  );
in
pkgs.symlinkJoin {
  name = "cg-perl-5.34.0";
  paths = [ perl ];
}
