#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir dart "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/Answer.dart"
	HOME=$dir "$CG_DART/bin/dart" --snapshot-kind=kernel --snapshot="$dir/Answer.dill" "$dir/Answer.dart" </dev/null >&2
	write_launcher "$dir" "$CG_DART/bin/dart" "$dir/Answer.dill"
fi
echo "$dir/run"
