{ pkgs, ... }:
let
  version = "3.0.8";
  dist = pkgs.fetchzip {
    url = "https://archive.apache.org/dist/groovy/${version}/distribution/apache-groovy-binary-${version}.zip";
    sha256 = "sha256-Vj5yNrC2AcSNs9MWA8K40KbHF0go0LxoKV8erSosMfA=";
  };
in
pkgs.runCommand "cg-groovy-${version}" { nativeBuildInputs = [ pkgs.makeWrapper ]; } ''
  mkdir -p $out/bin
  ln -s ${dist}/lib $out/lib
  for tool in groovy groovyc; do
    makeWrapper ${dist}/bin/$tool $out/bin/$tool --set JAVA_HOME ${pkgs.jdk8} --prefix PATH : ${pkgs.jdk8}/bin
  done
  ln -s ${pkgs.jdk8}/bin/java $out/bin/java
''
