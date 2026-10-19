{ fetchNixpkgs, ... }:
let
  pkgs = fetchNixpkgs {
    rev = "6233296137da293de6719c7b0cf9435a67dadc8c";
    sha256 = "1kll3bs7f566m059d52x4z7za1yy17gv09l800zw4fdlxryslglg";
  };
  npmTarball =
    name: version: sha256:
    pkgs.fetchurl {
      url = "https://registry.npmjs.org/${name}/-/${baseNameOf name}-${version}.tgz";
      inherit sha256;
    };
  typesNode = npmTarball "@types/node" "22.5.5" "1b62zssz250g1jbbsynyqlkbjcr627zazq0a7scbq33q0n9haaxw";
  undiciTypes = npmTarball "undici-types" "6.19.8" "0674q2m7xsg00rmz0w3v9llxm6qn25nccd901v0fqnwl5dqdjckc";
  cgTypes = pkgs.runCommand "typescript-cg-types" { } ''
    modules=$out/lib/node_modules
    types=$out/lib/cg-types
    mkdir -p $modules/@types/node $modules/undici-types $types/readline $types/assert
    tar -xzf ${typesNode} -C $modules/@types/node --strip-components=1
    tar -xzf ${undiciTypes} -C $modules/undici-types --strip-components=1
    echo 'declare function readline(): string' >$types/readline/index.d.ts
    cat >$types/assert/index.d.ts <<'EOF'
    import nodeAssert from 'assert'

    declare global {
      var assert: typeof nodeAssert;
    }
    EOF
  '';
in
pkgs.symlinkJoin {
  name = "typescript-cg";
  paths = [
    pkgs.nodejs_20
    pkgs.typescript
    cgTypes
  ];
}
