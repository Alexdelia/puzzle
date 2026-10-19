#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir ruby "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.rb"
	write_launcher "$dir" "$CG_RUBY/bin/ruby" "$dir/main.rb"
fi
echo "$dir/run"
