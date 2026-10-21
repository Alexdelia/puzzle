#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
main=$(main_class)
dir=$(cache_dir scala "$src" "$main")
if [[ ! -x $dir/run ]]; then
	rm -rf "$dir/classes"
	mkdir -p "$dir/classes"
	cp "$src" "$dir/$main.scala"
	"$CG_SCALA/bin/scalac" -nowarn -d "$dir/classes" "$dir/$main.scala" </dev/null >&2
	write_launcher "$dir" "$CG_SCALA/bin/scala" -cp "$dir/classes" "$main"
fi
echo "$dir/run"
