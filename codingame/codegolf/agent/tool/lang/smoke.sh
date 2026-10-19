#!/usr/bin/env bash
set -euo pipefail

lang=${1:?usage: smoke.sh <lang>}
lang_dir=$(cd "$(dirname "$0")/$lang" && pwd)
src=$(find "$lang_dir/smoke" -name 'main.*' | head -1)
launcher=$("$lang_dir/build.sh" "$src")
actual=$("$launcher" <"$lang_dir/smoke/input")
expected=$(<"$lang_dir/smoke/expected")
if [[ $actual == "$expected" ]]; then
	echo "smoke $lang ok"
else
	echo "smoke $lang failed: expected '$expected' got '$actual'" >&2
	exit 1
fi
