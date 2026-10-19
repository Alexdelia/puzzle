#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir java "$src")
if [[ ! -x $dir/run ]]; then
	mkdir -p "$dir/src" "$dir/out"
	cp "$src" "$dir/src/Solution.java"
	"$CG_JAVA/bin/javac" -encoding UTF-8 -nowarn -d "$dir/out" "$dir/src/Solution.java" >&2
	write_launcher "$dir" /usr/bin/env LC_ALL=C.UTF-8 "$CG_JAVA/bin/java" -Xmx300m -XX:ActiveProcessorCount=1 -cp "$dir/out" Solution
fi
echo "$dir/run"
