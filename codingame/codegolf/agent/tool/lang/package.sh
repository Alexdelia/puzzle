#!/usr/bin/env bash
set -euo pipefail

lang=${1:?usage: package.sh <lang>, builds agent/tool/lang/<lang>/package.nix even untracked, prints its store path}
lang_dir=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$lang_dir/../../.." && pwd)
repo=$(git -C "$root" rev-parse --show-toplevel)
subdir=$(git -C "$root" rev-parse --show-prefix)
nix build --impure --no-link --print-out-paths --expr "
	import $lang_dir/$lang/package.nix
		(builtins.getFlake \"git+file://$repo?dir=${subdir%/}\").legacyPackages.\${builtins.currentSystem}.langArgs
"
