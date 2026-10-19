{ pkgs, ... }:
let
  version = "1.7.10";
  dist = pkgs.fetchzip {
    url = "https://github.com/JetBrains/kotlin/releases/download/v${version}/kotlin-compiler-${version}.zip";
    sha256 = "sha256-q8tRVft2kyCPUk5HuZS7Bqq6ds60ZhLAcqBkqmFOlF4=";
  };
  maven = path: sha256: pkgs.fetchurl {
    url = "https://repo1.maven.org/maven2/${path}";
    inherit sha256;
  };
  libs = {
    "kotlinx-coroutines-core.jar" = maven "org/jetbrains/kotlinx/kotlinx-coroutines-core-jvm/1.6.4/kotlinx-coroutines-core-jvm-1.6.4.jar" "sha256-wkyLsnuzIMSpOHFQGn5eDGFgdjiQexl672dVE9TIIL4=";
    "kotlinx-serialization-core-jvm.jar" = maven "org/jetbrains/kotlinx/kotlinx-serialization-core-jvm/1.4.0/kotlinx-serialization-core-jvm-1.4.0.jar" "sha256-nt+rEpzQohnwagGluvbAOgm124taO+xN2VBx7dYdYnE=";
    "kotlinx-serialization-json-jvm.jar" = maven "org/jetbrains/kotlinx/kotlinx-serialization-json-jvm/1.4.0/kotlinx-serialization-json-jvm-1.4.0.jar" "sha256-ocNE0OIfFP2C7bzaw3TPFg1yiUWdz3Xexoip+9zmoJk=";
  };
in
pkgs.runCommand "cg-kotlin-${version}" { nativeBuildInputs = [ pkgs.makeWrapper ]; } ''
  mkdir -p $out/bin $out/lib
  ln -s ${dist}/lib/* $out/lib/
  ln -s $out/lib/annotations-13.0.jar $out/lib/annotations.jar
  ${pkgs.lib.concatStrings (pkgs.lib.mapAttrsToList (name: jar: "ln -s ${jar} $out/lib/${name}\n") libs)}
  makeWrapper ${dist}/bin/kotlinc $out/bin/kotlinc --set JAVA_HOME ${pkgs.jdk8} --prefix PATH : ${pkgs.jdk8}/bin
  ln -s ${pkgs.jdk21}/bin/java $out/bin/java
''
