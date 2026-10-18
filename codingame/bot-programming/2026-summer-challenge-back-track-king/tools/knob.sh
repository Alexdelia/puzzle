#!/usr/bin/env bash
set -euo pipefail

bot=${BOT:-marshal}
foe=${FOE:-sentinel}
games=${GAMES:-200}
seeds=${SEEDS:-7000000 7100000}
root=$(cd "$(dirname "$0")/.." && pwd)
bin=$root/target/release
pinned="env BTK_THINK_MS=3000 BTK_PLOTS=0"
case $bot in /*) ;; *) bot=$bin/$bot ;; esac
case $foe in /*) ;; *) foe=$bin/$foe ;; esac
snap=$(mktemp -d "${TMPDIR:-/tmp}/knob.XXXXXX")
cp "$bot" "$snap/bot" && bot=$snap/bot
cp "$foe" "$snap/foe" && foe=$snap/foe
trap 'rm -rf "$snap"' EXIT

run() {
	local spec=$1 total_w=0 total_g=0 mine=0 yours=0 detail=""
	for seed in $seeds; do
		out=$("$bin/arena" --games "$games" --seed "$seed" --quiet --no-timings \
			--p1 "$pinned $spec $bot" --p2 "$pinned $foe" 2>/dev/null)
		w=$(echo "$out" | sed -n 's/^p1 wins: \([0-9]*\) .*/\1/p')
		read -r a b < <(echo "$out" | sed -n 's/^mean points: p1 \([0-9.]*\)  p2 \([0-9.]*\)/\1 \2/p')
		total_w=$((total_w + w)); total_g=$((total_g + games))
		mine=$(awk -v x=$mine -v y="$a" 'BEGIN{print x+y}')
		yours=$(awk -v x=$yours -v y="$b" 'BEGIN{print x+y}')
		detail="$detail $(awk -v w=$w -v g=$games 'BEGIN{printf "%.0f", 100*w/g}')"
	done
	printf '%-52s %5.1f%% win  %5.2f%% pts  [%s ]\n' "${spec:-<defaults>}" \
		"$(awk -v w=$total_w -v g=$total_g 'BEGIN{print 100*w/g}')" \
		"$(awk -v a=$mine -v b=$yours 'BEGIN{print 100*a/(a+b)}')" "$detail"
}

if [ $# -eq 0 ]; then
	while IFS= read -r line; do run "$line"; done
else
	for spec in "$@"; do run "$spec"; done
fi
