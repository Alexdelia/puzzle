#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir bash "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.sh"
	write_launcher "$dir" "$CG_BASH/bin/env" -i PATH="$CG_BASH/bin:${CG_LUA:?CG_LUA unset, enter the codegolf dev shell}/bin" LC_ALL=en_US.UTF-8 LOCALE_ARCHIVE="$CG_BASH/lib/locale/locale-archive" "$CG_BASH/bin/bash" "$dir/main.sh"
fi
echo "$dir/run"
