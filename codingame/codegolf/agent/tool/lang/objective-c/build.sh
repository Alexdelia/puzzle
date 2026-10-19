#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir objective-c "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.m"
	NIX_HARDENING_ENABLE= "$CG_OBJECTIVE_C/bin/clang" -w -O2 -D_FORTIFY_SOURCE=2 -fno-PIE -no-pie \
		-fobjc-runtime=gnustep-1.8 -fblocks -fobjc-exceptions -fexceptions -pthread \
		-DGNUSTEP -DGNUSTEP_BASE_LIBRARY=1 -DGNU_RUNTIME=1 -D_NONFRAGILE_ABI=1 -D_NATIVE_OBJC_EXCEPTIONS \
		-I"$CG_OBJECTIVE_C/include" -o "$dir/main" "$dir/main.m" \
		-L"$CG_OBJECTIVE_C/lib" -Wl,-rpath,"$CG_OBJECTIVE_C/lib" -lgnustep-base -lobjc -lm -lpthread -ldl
	write_launcher "$dir" "$CG_OBJECTIVE_C/bin/setarch" -R "$dir/main"
fi
echo "$dir/run"
