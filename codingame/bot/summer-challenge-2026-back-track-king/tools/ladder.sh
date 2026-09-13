#!/usr/bin/env bash
set -euo pipefail

candidate=${1:?usage: ladder.sh CANDIDATE_SPEC [games] [seed] [opponents...]}
games=${2:-200}
seed=${3:-7000000}
shift $(( $# < 3 ? $# : 3 )) || true
opponents=("$@")
if [ ${#opponents[@]} -eq 0 ]; then
	opponents=(sentinel vanguard squire warden probe bulwark champ corridor trunk)
fi
root=$(cd "$(dirname "$0")/.." && pwd)
bin=$root/target/release

spec_of() {
	case $1 in
		sentinel|vanguard|warden|probe|bulwark|rampart)
			echo "env BTK_THINK_MS=3000 BTK_PLOTS=0 $bin/$1" ;;
		*) echo "$bin/$1" ;;
	esac
}

printf '%-10s %7s %7s %7s  %s\n' opponent winA winB mean share
for foe in "${opponents[@]}"; do
	rates=()
	shares=()
	for offset in 0 100000; do
		out=$("$bin/arena" --games "$games" --seed $((seed + offset)) --quiet --no-timings \
			--p1 "$candidate" --p2 "$(spec_of "$foe")" 2>/dev/null)
		rate=$(echo "$out" | sed -n 's/^p1 wins: [0-9]* (\([0-9.]*\)%).*/\1/p')
		share=$(echo "$out" | sed -n 's/^mean points: p1 \([0-9.]*\)  p2 \([0-9.]*\)/\1 \2/p' \
			| awk '{ printf "%.1f", 100 * $1 / ($1 + $2) }')
		rates+=("$rate")
		shares+=("$share")
	done
	mean=$(awk -v a="${rates[0]}" -v b="${rates[1]}" 'BEGIN { printf "%.1f", (a + b) / 2 }')
	printf '%-10s %6s%% %6s%% %6s%%  %s/%s\n' "$foe" "${rates[0]}" "${rates[1]}" "$mean" "${shares[0]}" "${shares[1]}"
done
