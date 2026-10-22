unset NIX_CFLAGS_COMPILE NIX_CFLAGS_COMPILE_FOR_BUILD NIX_CFLAGS_LINK NIX_LDFLAGS NIX_LDFLAGS_FOR_BUILD

build_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/state/build/lang

toolchain_var() {
	local name=${1//-/_}
	echo "CG_${name^^}"
}

cache_dir() {
	local lang=$1 src=$2
	shift 2
	local toolchain_var_name
	toolchain_var_name=$(toolchain_var "$lang")
	local toolchain=${!toolchain_var_name:?$toolchain_var_name unset, enter the codegolf dev shell}
	local key
	key=$(cat "$src" "${BASH_SOURCE[1]}" "${BASH_SOURCE[0]}" <(echo "$toolchain $CG_DASH $CG_BWRAP $CG_PERL $CG_PYTHON $CG_LUA $*") | sha256sum | cut -c1-32)
	local dir=$build_root/$lang/$key
	mkdir -p "$dir"
	echo "$dir"
}

main_class() {
	if [[ ${CG_PUZZLE_KIND:-io} == interactive ]]; then
		echo Player
	else
		echo Solution
	fi
}

write_launcher() {
	local dir=$1
	shift
	local env
	env=$(readlink -f /usr/bin/env)
	printf '#!/bin/sh\nexec %s/bin/bwrap --dev-bind / / --tmpfs /bin --symlink %s/bin/dash /bin/sh --tmpfs /usr/bin --symlink %s /usr/bin/env --symlink %s/bin/perl /usr/bin/perl --symlink %s/bin/python3 /usr/bin/python3 --setenv PATH "%s/bin:%s/bin:$PATH" --tmpfs /tmp --chdir /tmp --die-with-parent %s\n' \
		"${CG_BWRAP:?CG_BWRAP unset, enter the codegolf dev shell}" "${CG_DASH:?CG_DASH unset, enter the codegolf dev shell}" "$env" \
		"${CG_PERL:?CG_PERL unset, enter the codegolf dev shell}" "${CG_PYTHON:?CG_PYTHON unset, enter the codegolf dev shell}" "$CG_PERL" "${CG_LUA:?CG_LUA unset, enter the codegolf dev shell}" "$*" >"$dir/run.tmp"
	chmod +x "$dir/run.tmp"
	mv "$dir/run.tmp" "$dir/run"
}
