#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir kotlin "$src")
if [[ ! -x $dir/run ]]; then
	lib=$CG_KOTLIN/lib
	jars=(kotlin-stdlib-jdk8.jar kotlin-stdlib-jdk7.jar kotlin-stdlib.jar annotations.jar kotlinx-coroutines-core.jar kotlinx-serialization-core-jvm.jar kotlinx-serialization-json-jvm.jar)
	libpath=$(printf '%s:' "${jars[@]/#/$lib/}")
	rm -rf "$dir/classes"
	mkdir -p "$dir/classes"
	cp "$src" "$dir/Answer.kt"
	"$CG_KOTLIN/bin/kotlinc" -nowarn -cp "${libpath%:}" -d "$dir/classes" "$dir/Answer.kt" </dev/null >&2
	write_launcher "$dir" "$CG_KOTLIN/bin/java" -Xms64m -Xmx450m -Xverify:none -Djava.awt.headless=true -Djava.io.tmpdir=/tmp -ea -cp "$libpath$dir/classes" AnswerKt
fi
echo "$dir/run"
