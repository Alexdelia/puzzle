#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir cpp "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.cpp"
	NIX_HARDENING_ENABLE= "$CG_CPP/bin/g++" -std=gnu++20 -w -fPIE -pie -o "$dir/main" "$dir/main.cpp" -lm -lpthread -ldl -lcrypt
	write_launcher "$dir" "$CG_CPP/bin/setarch" -R "$dir/main"
fi
echo "$dir/run"
