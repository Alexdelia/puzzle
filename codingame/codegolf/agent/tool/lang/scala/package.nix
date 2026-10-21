{ pkgs, ... }:
let
  version = "2.13.5";
  maven = path: sha256: pkgs.fetchurl {
    url = "https://repo1.maven.org/maven2/${path}";
    inherit sha256;
  };
  scalaJar = artifact: sha256: maven "org/scala-lang/scala-${artifact}/${version}/scala-${artifact}-${version}.jar" sha256;
  jars = [
    { name = "jline-3.19.0.jar"; jar = maven "org/jline/jline/3.19.0/jline-3.19.0.jar" "sha256-yZ3c+lQxyriNHNQP1jvsarWj/i6Dh3BRGYU5r2ZZKkY="; }
    { name = "jna-5.3.1.jar"; jar = maven "net/java/dev/jna/jna/5.3.1/jna-5.3.1.jar" "sha256-ActQXAaY0Pes81JMfnOst9xCSluuXpyGzkQHWrMrxO4="; }
    { name = "scala-compiler.jar"; jar = scalaJar "compiler" "sha256-6nQj87w2c4RdbRxkM1pGRau6CwR4rgDhWXmRWCb/YRY="; }
    { name = "scala-library.jar"; jar = scalaJar "library" "sha256-Uqr+744NEEQzMpsbwxRj0bSp4rjyT4VDLIz67Z+tJYc="; }
    { name = "scalap-${version}.jar"; jar = maven "org/scala-lang/scalap/${version}/scalap-${version}.jar" "sha256-d9tERoLvir8G7OOOHKqDC9Y9UmkWaYVAmSUry2VHfmo="; }
    { name = "scala-reflect.jar"; jar = scalaJar "reflect" "sha256-gIxEuK2zIF6R1Be/V0BnFcolCK1pUvbiEy/4CZt4vXM="; }
  ];
  bootClassPath = pkgs.lib.concatMapStringsSep ":" (j: "${placeholder "out"}/lib/${j.name}") jars;
  externalStub = "${placeholder "out"}/lib/external-stub";
  launcher = name: jdk: classPath: mainClass: ''
    cat >$out/bin/${name} <<EOF
    #!/bin/sh
    exec ${jdk}/bin/java -XX:ActiveProcessorCount=1 -Xmx256M -Xms32M -Xbootclasspath/a:${bootClassPath} -classpath "${classPath}" -Dscala.boot.class.path=${bootClassPath} -Dscala.home=$out -Dscala.usejavacp=true ${mainClass} "\$@"
    EOF
    chmod +x $out/bin/${name}
  '';
in
pkgs.runCommand "cg-scala-${version}" { } ''
  mkdir -p $out/bin $out/lib
  ${pkgs.lib.concatMapStrings (j: "ln -s ${j.jar} $out/lib/${j.name}\n") jars}
  ln -s ${pkgs.jdk21}/bin/java $out/bin/java
  mkdir -p stub/io/circe ${externalStub}
  echo 'package io.circe; public class Stub {}' >stub/io/circe/Stub.java
  ${pkgs.jdk17}/bin/javac --release 8 -d ${externalStub} stub/io/circe/Stub.java
  ${launcher "scalac" pkgs.jdk17 externalStub "scala.tools.nsc.Main"}
  ${launcher "scala" pkgs.jdk21 "" "scala.tools.nsc.MainGenericRunner"}
''
