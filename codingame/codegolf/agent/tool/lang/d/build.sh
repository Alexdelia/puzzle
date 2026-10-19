#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir d "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.d"
	NIX_HARDENING_ENABLE= "$CG_D/bin/dmd" -od"$dir" -of"$dir/main" "$dir/main.d"
	write_launcher "$dir" "$dir/main"
fi
echo "$dir/run"
