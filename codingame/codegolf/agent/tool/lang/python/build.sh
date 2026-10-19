#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir python "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.py"
	write_launcher "$dir" "$CG_PYTHON/bin/python3" "$dir/main.py"
fi
echo "$dir/run"
