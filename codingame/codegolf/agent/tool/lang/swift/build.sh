#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir swift "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.swift"
	"$CG_SWIFT/bin/swiftc" -O -suppress-warnings -Xcc -isystem"$CG_SWIFT/sysinclude" \
		-module-cache-path "$dir/module-cache" -o "$dir/main" "$dir/main.swift"
	write_launcher "$dir" "$dir/main"
fi
echo "$dir/run"
