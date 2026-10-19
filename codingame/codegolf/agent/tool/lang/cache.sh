build_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/state/build/lang

toolchain_var() {
	local name=${1//-/_}
	echo "CG_${name^^}"
}

cache_dir() {
	local lang=$1 src=$2
	local toolchain_var_name
	toolchain_var_name=$(toolchain_var "$lang")
	local toolchain=${!toolchain_var_name:?$toolchain_var_name unset, enter the codegolf dev shell}
	local key
	key=$(cat "$src" "${BASH_SOURCE[1]}" <(echo "$toolchain") | sha256sum | cut -c1-32)
	local dir=$build_root/$lang/$key
	mkdir -p "$dir"
	echo "$dir"
}

write_launcher() {
	local dir=$1
	shift
	printf '#!/bin/sh\nexec %s\n' "$*" >"$dir/run.tmp"
	chmod +x "$dir/run.tmp"
	mv "$dir/run.tmp" "$dir/run"
}
