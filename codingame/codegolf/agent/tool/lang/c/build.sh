#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir c "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.c"
	NIX_HARDENING_ENABLE= "$CG_C/bin/gcc" -std=gnu17 -w -fPIE -pie -o "$dir/main" "$dir/main.c" -lm -lpthread -ldl -lcrypt
	write_launcher "$dir" "$CG_C/bin/setarch" -R "$dir/main"
fi
echo "$dir/run"
