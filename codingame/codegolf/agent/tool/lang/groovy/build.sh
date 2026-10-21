#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir groovy "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/Answer.groovy"
	rm -rf "$dir/classes"
	"$CG_GROOVY/bin/groovyc" -d "$dir/classes" "$dir/Answer.groovy" >&2
	write_launcher "$dir" "$CG_GROOVY/bin/java" -Xmx290m -cp "'$dir/classes:$CG_GROOVY/lib/*'" Answer
fi
echo "$dir/run"
