#!/usr/bin/env bash
set -euo pipefail
usage="usage: selfplay.sh <bot> <first seed> <last seed> <output dir> (records both seats' stdin and answers as s<seed>-<bot>0/1.in/.out)"
bot=${1:?$usage}; first=${2:?$usage}; last=${3:?$usage}; out=${4:?$usage}
root=$(cd "$(dirname "$0")/../.." && pwd)
name=$(basename "$bot")
mkdir -p "$out"
seq "$first" "$last" | xargs -P "${JOBS:-8}" -I{} sh -c '
	stem="$1/s{}-$2"
	"$3/target/release/trace" --league 3 --seed {} \
		--p1 "sh -c '\''tee $stem""0.in | $4 | tee $stem""0.out'\''" \
		--p2 "sh -c '\''tee $stem""1.in | $4 | tee $stem""1.out'\''" > /dev/null 2>&1
' _ "$out" "$name" "$root" "$bot"
ls "$out" | wc -l
