#!/usr/bin/env bash
set -euo pipefail
usage="usage: dump.sh <output dir> <stream dir>... (dumps every <game>-<nick>.in with its .out, skipping Alexdelia)"
out=${1:?$usage}; shift
root=$(cd "$(dirname "$0")/../.." && pwd)
bot=$root/target/release/nova
mkdir -p "$out"
for dir in "$@"; do
	for stream in "$dir"/*.in; do
		name=$(basename "$stream" .in)
		[[ $name == *-Alexdelia ]] && continue
		[[ -s ${stream%.in}.out ]] || continue
		echo "$stream $out/$name.bin"
	done
done | xargs -P 8 -L 1 sh -c 'NOVA_DUMP="$1" NOVA_ANSWERS="${0%.in}.out" '"$bot"' < "$0" > /dev/null'
ls "$out" | wc -l
