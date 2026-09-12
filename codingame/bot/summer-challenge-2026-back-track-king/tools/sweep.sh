#!/usr/bin/env bash
set -euo pipefail

bot=${BOT:-squire}
foe=${FOE:-sentinel}
games=${GAMES:-100}
seeds=${SEEDS:-7000000 7100000}
root=$(cd "$(dirname "$0")/.." && pwd)
bin=$root/target/release
case $bot in /*) ;; *) bot=$bin/$bot ;; esac
case $foe in /*) ;; *) foe=$bin/$foe ;; esac
snap=$(mktemp -d "${TMPDIR:-/tmp}/sweep.XXXXXX")
cp "$bot" "$snap/bot" && bot=$snap/bot
cp "$foe" "$snap/foe" && foe=$snap/foe
trap 'rm -rf "$snap"' EXIT

run() {
	local env_spec=$1
	local total_w=0 total_g=0 shares=""
	for seed in $seeds; do
		out=$("$bin/arena" --games "$games" --seed "$seed" --quiet --no-timings \
			--p1 "env $env_spec $bot" --p2 "$foe" 2>/dev/null)
		w=$(echo "$out" | sed -n 's/^p1 wins: \([0-9]*\) .*/\1/p')
		share=$(echo "$out" | sed -n 's/^mean points: p1 \([0-9.]*\)  p2 \([0-9.]*\)/\1 \2/p' \
			| awk '{ printf "%.1f", 100 * $1 / ($1 + $2) }')
		total_w=$((total_w + w))
		total_g=$((total_g + games))
		shares="$shares $share"
	done
	printf '%-50s %5.1f%%  share%s\n' "$env_spec" "$(awk -v w=$total_w -v g=$total_g 'BEGIN{print 100*w/g}')" "$shares"
}

if [ $# -eq 0 ]; then
	while IFS= read -r line; do
		[ -z "$line" ] && continue
		run "$line"
	done
else
	for spec in "$@"; do
		run "$spec"
	done
fi
