#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir perl "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.pl"
	write_launcher "$dir" "$CG_PERL/bin/perl" "$dir/main.pl"
fi
echo "$dir/run"
