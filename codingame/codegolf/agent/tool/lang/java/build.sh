#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
main=$(main_class)
dir=$(cache_dir java "$src" "$main")
if [[ ! -x $dir/run ]]; then
	mkdir -p "$dir/src" "$dir/out"
	cp "$src" "$dir/src/$main.java"
	"$CG_JAVA/bin/javac" -encoding UTF-8 -nowarn -d "$dir/out" "$dir/src/$main.java" >&2
	write_launcher "$dir" /usr/bin/env LC_ALL=C.UTF-8 "$CG_JAVA/bin/java" -Xmx300m -XX:ActiveProcessorCount=1 -cp "$dir/out" "$main"
fi
echo "$dir/run"
