#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir pascal "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.pas"
	"$CG_PASCAL/bin/fpc" -v0 -FE"$dir" -o"$dir/main" "$dir/main.pas" >"$dir/compile.log" || {
		cat "$dir/compile.log" >&2
		exit 1
	}
	write_launcher "$dir" "$dir/main"
fi
echo "$dir/run"
