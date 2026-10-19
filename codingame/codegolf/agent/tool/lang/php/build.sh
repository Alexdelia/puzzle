#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir php "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.php"
	write_launcher "$dir" "$CG_PHP/bin/php" -n \
		-d short_open_tag=1 \
		-d error_reporting=32767 \
		-d display_errors=stderr \
		-d memory_limit=512M \
		-d disable_functions=ini_set \
		"$dir/main.php"
fi
echo "$dir/run"
