#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir clojure "$src")
if [[ ! -x $dir/run ]]; then
	rm -rf "$dir/src" "$dir/classes"
	mkdir -p "$dir/src" "$dir/classes"
	cp "$src" "$dir/src/Solution.clj"
	(cd "$dir/classes" && "$CG_CLOJURE/bin/clojure-compile" "$dir/src" "$dir/classes" <"$dir" >&2)
	write_launcher "$dir" "$CG_CLOJURE/bin/clojure-run" "$dir/classes"
fi
echo "$dir/run"
