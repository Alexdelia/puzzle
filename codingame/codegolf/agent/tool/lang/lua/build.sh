#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir lua "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.lua"
	write_launcher "$dir" "$CG_LUA/bin/lua" "$dir/main.lua"
fi
echo "$dir/run"
