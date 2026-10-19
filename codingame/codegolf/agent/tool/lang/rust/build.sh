#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir rust "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.rs"
	"$CG_RUST/bin/rustc" --edition 2024 -C opt-level=3 -C linker="$CG_RUST/bin/cc" -A warnings \
		-o "$dir/main" "$dir/main.rs"
	write_launcher "$dir" /usr/bin/env RUST_BACKTRACE=1 "$CG_RUST/bin/taskset" -c 0 "$dir/main"
fi
echo "$dir/run"
