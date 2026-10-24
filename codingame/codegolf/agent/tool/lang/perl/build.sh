#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir perl "$src")
if [[ ! -x $dir/run ]]; then
	{
		cat "$src"
		[[ -z $(tail -c 1 "$src") ]] || echo
	} >"$dir/main.pl"
	write_launcher "$dir" "$CG_PERL/bin/perl" "$dir/main.pl"
fi
echo "$dir/run"
