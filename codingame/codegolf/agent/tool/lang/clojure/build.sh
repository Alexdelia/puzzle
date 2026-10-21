#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
main=$(main_class)
dir=$(cache_dir clojure "$src" "$main")
if [[ ! -x $dir/run ]]; then
	rm -rf "$dir/src" "$dir/classes"
	mkdir -p "$dir/src" "$dir/classes"
	cp "$src" "$dir/src/$main.clj"
	(cd "$dir/classes" && "$CG_CLOJURE/bin/clojure-compile" "$dir/src" "$dir/classes" "$main" <"$dir" >&2)
	if [[ $main == Player ]]; then
		write_launcher "$dir" "$CG_CLOJURE/bin/clojure-run-interactive" "$dir/classes"
	else
		write_launcher "$dir" "$CG_CLOJURE/bin/clojure-run" "$dir/classes" "$main"
	fi
fi
echo "$dir/run"
