#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
polyfill=$(cd "$(dirname "$0")/../javascript/polyfill" && pwd)/polyfill.js
dir=$(cache_dir typescript "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/Answer.ts"
	types=$CG_TYPESCRIPT/lib/cg-types
	(cd "$dir" && "$CG_TYPESCRIPT/bin/node" "$CG_TYPESCRIPT/lib/node_modules/typescript/bin/tsc" \
		--target esnext \
		--lib esnext \
		--esModuleInterop \
		--inlineSourceMap \
		--types "$types/assert,$types/readline,$CG_TYPESCRIPT/lib/node_modules/@types/node" \
		--baseUrl ./ \
		--module commonjs \
		--allowJS \
		--outDir "$dir/build/" \
		Answer.ts >&2)
	mv "$dir/build/Answer.js" "$dir/Answer.js"
	write_launcher "$dir" /usr/bin/env -C /tmp TZ=UTC LC_ALL=en_US.UTF-8 HOME=/tmp \
		"$CG_TYPESCRIPT/bin/node" -r "$polyfill" "$dir/Answer.js"
fi
echo "$dir/run"
