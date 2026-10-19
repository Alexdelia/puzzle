#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
polyfill=$(cd "$(dirname "$0")/polyfill" && pwd)/polyfill.js
dir=$(cache_dir javascript "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/Answer.js"
	write_launcher "$dir" /usr/bin/env -C /tmp TZ=UTC LC_ALL=en_US.UTF-8 HOME=/tmp \
		"$CG_JAVASCRIPT/bin/node" -r "$polyfill" "$dir/Answer.js"
fi
echo "$dir/run"
