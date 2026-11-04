#!/usr/bin/env bash
set -euo pipefail
usage="usage: duel.sh <weights.bin> <opponent command> [games] [extra nova env, e.g. NOVA_SAMPLE=7]"
weights=${1:?$usage}; opponent=${2:?$usage}; games=${3:-40}; extra=${4:-}
root=$(cd "$(dirname "$0")/../.." && pwd)
bot="$root/target/nova-duel-$$"; cp "$root/target/release/nova" "$bot"; trap 'rm -f "$bot"' EXIT
"$root/target/release/arena" --league 3 --games "$games" --threads "${THREADS:-4}" --seed "${SEED:-900000}" \
	--p1 "env NOVA_WEIGHTS=$weights $extra $bot" --p2 "$opponent" 2>&1 | grep -E "^games|^mean"
