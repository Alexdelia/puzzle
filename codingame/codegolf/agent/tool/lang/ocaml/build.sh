#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir ocaml "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.ml"
	PATH=$CG_OCAML/bin:$PATH "$CG_OCAML/bin/ocamlopt" -o "$dir/main" "$dir/main.ml" >&2
	write_launcher "$dir" "$dir/main"
fi
echo "$dir/run"
