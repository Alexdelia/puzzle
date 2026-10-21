{ pkgs, ... }:
let
  maven = path: sha256: pkgs.fetchurl {
    url = "https://repo1.maven.org/maven2/${path}";
    inherit sha256;
  };
  jars = [
    (maven "org/clojure/clojure/1.11.1/clojure-1.11.1.jar" "sha256-I4G26UI6tGUVFFWUSQPROlYkPWAGuRlK/Bv0+HEMtN4=")
    (maven "org/clojure/spec.alpha/0.3.218/spec.alpha-0.3.218.jar" "sha256-Z+yJjrVcZqlXpVJ53YXRN2u5lL2HZosrDeHrO5foquA=")
    (maven "org/clojure/core.specs.alpha/0.2.62/core.specs.alpha-0.2.62.jar" "sha256-Bu6owHC75FwVhWfkQ0OWgbyMRukSNBT4G/oyukLWy8g=")
  ];
  classpath = pkgs.lib.concatStringsSep ":" jars;
  bridge = pkgs.writeText "Bridge.java" ''
    public class Bridge {
      public static void main(String[] args) throws Exception {
        clojure.java.api.Clojure.var("clojure.core", "require").invoke(clojure.java.api.Clojure.read("Player"));
        clojure.java.api.Clojure.var("Player", "-main").invoke(null);
      }
    }
  '';
in
pkgs.runCommand "cg-clojure-1.11.1" { } ''
  mkdir -p $out/bin $out/share/java $out/share/bridge
  ln -s ${pkgs.jdk8}/bin/java $out/bin/java8
  ln -s ${pkgs.jdk21}/bin/java $out/bin/java
  ${pkgs.lib.concatMapStrings (jar: "ln -s ${jar} $out/share/java/${jar.name}\n") jars}
  cp ${bridge} Bridge.java
  ${pkgs.jdk8}/bin/javac -cp ${classpath} -d $out/share/bridge Bridge.java
  cat >$out/bin/clojure-compile <<EOF
  #!/bin/sh
  exec ${pkgs.jdk8}/bin/java -Xmx450m -Xverify:none -Dclojure.compile.path="\$2" -cp ${classpath}:"\$1":"\$2" clojure.lang.Compile "\$3"
  EOF
  cat >$out/bin/clojure-run <<EOF
  #!/bin/sh
  exec ${pkgs.jdk21}/bin/java -cp ${classpath}:"\$1"/ "\$2"
  EOF
  cat >$out/bin/clojure-run-interactive <<EOF
  #!/bin/sh
  exec ${pkgs.jdk8}/bin/java -cp ${classpath}:$out/share/bridge:"\$1"/ Bridge
  EOF
  cat >$out/bin/clojure <<EOF
  #!/bin/sh
  exec ${pkgs.jdk21}/bin/java -cp ${classpath} clojure.main "\$@"
  EOF
  chmod +x $out/bin/clojure-compile $out/bin/clojure-run $out/bin/clojure-run-interactive $out/bin/clojure
''
