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
	key=$(cat "$src" "${BASH_SOURCE[1]}" "${BASH_SOURCE[0]}" <(echo "$toolchain $CG_DASH $CG_BWRAP $CG_PERL $CG_PYTHON $CG_LUA $CG_BASH $CG_PHP $CG_RUBY $CG_JAVASCRIPT $*") | sha256sum | cut -c1-32)
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
	local missing="unset, enter the codegolf dev shell"
	local sandbox=(
		--dev-bind / / --tmpfs /bin --tmpfs /usr --bind /usr/share /usr/share --dir /usr/bin --dir /usr/local/bin
		--symlink "${CG_DASH:?CG_DASH $missing}/bin/dash" /bin/sh
		--symlink "${CG_BASH:?CG_BASH $missing}/bin/bash" /bin/bash
		--symlink "$(readlink -f /usr/bin/env)" /usr/bin/env
		--symlink "${CG_PERL:?CG_PERL $missing}/bin/perl" /usr/bin/perl
		--symlink "${CG_PYTHON:?CG_PYTHON $missing}/bin/python3" /usr/bin/python3
		--symlink "${CG_LUA:?CG_LUA $missing}/bin/lua" /usr/bin/lua
		--symlink "${CG_PHP:?CG_PHP $missing}/bin/php" /usr/bin/php
		--symlink "$CG_PHP/bin/php" /bin/php
		--symlink "${CG_RUBY:?CG_RUBY $missing}/bin/ruby" /usr/bin/ruby
		--symlink "$CG_RUBY/bin/ruby" /usr/local/bin/ruby
		--symlink "${CG_JAVASCRIPT:?CG_JAVASCRIPT $missing}/bin/node" /usr/bin/node
		--setenv PATH "/usr/local/bin:/usr/bin:/bin:$PATH"
		--tmpfs /tmp --chdir /tmp --die-with-parent
	)
	printf '#!/bin/sh\nexec %s/bin/bwrap%s %s\n' "${CG_BWRAP:?CG_BWRAP $missing}" "$(printf ' %q' "${sandbox[@]}")" "$*" >"$dir/run.tmp"
	chmod +x "$dir/run.tmp"
	mv "$dir/run.tmp" "$dir/run"
}
