#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir haskell "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.hs"
	"$CG_HASKELL/bin/ghc" -v0 -outputdir "$dir/obj" -o "$dir/main" "$dir/main.hs" >&2
	write_launcher "$dir" "$dir/main"
fi
echo "$dir/run"
