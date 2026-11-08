#!/usr/bin/env bash
set -euo pipefail
usage="usage: watch.sh <run dir> <opponent command> [every N updates] [games]"
run=${1:?$usage}; opponent=${2:?$usage}; every=${3:-100}; games=${4:-40}
here=$(cd "$(dirname "$0")" && pwd)
while :; do
	for weights in "$run"/upd*.bin; do
		[[ -e $weights ]] || continue
		update=$(basename "$weights" .bin); update=$((10#${update#upd}))
		((update % every == 0)) || continue
		grep -q "^upd $update " "$run/evals.log" 2>/dev/null && continue
		result=$("$here/duel.sh" "$weights" "$opponent" "$games" | head -1)
		echo "upd $update $result" >> "$run/evals.log"
	done
	sleep 60
done
