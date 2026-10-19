#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir scala "$src")
if [[ ! -x $dir/run ]]; then
	rm -rf "$dir/classes"
	mkdir -p "$dir/classes"
	cp "$src" "$dir/Solution.scala"
	"$CG_SCALA/bin/scalac" -nowarn -d "$dir/classes" "$dir/Solution.scala" </dev/null >&2
	write_launcher "$dir" "$CG_SCALA/bin/scala" -cp "$dir/classes" Solution
fi
echo "$dir/run"
