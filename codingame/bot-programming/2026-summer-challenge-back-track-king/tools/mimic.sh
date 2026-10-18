#!/usr/bin/env bash
set -euo pipefail

candidate=${1:?usage: mimic.sh CANDIDATE_SPEC [filter]}
filter=${2:-}
root=$(cd "$(dirname "$0")/.." && pwd)
bin=$root/target/release

leaders="ziemekb|Indrill|RamzaBeoulve|Rikus"
total_shared=0
total_union=0
total_disrupt=0
total_turns=0
printf '%-44s %6s %6s %6s\n' log overlap ink turns
for log in "$root"/.doc/top_log/*/*/*-*; do
	name=$(basename "$log")
	case "$name" in init|*leaderboard*) continue ;; esac
	if [ -n "$filter" ] && ! echo "$name" | grep -q "$filter"; then continue; fi
	first=${name%%-*}
	second=${name#*-}
	side=""
	if echo "$second" | grep -Eq "^($leaders)$"; then side=1; fi
	if echo "$first" | grep -Eq "^($leaders)$"; then side=0; fi
	[ -z "$side" ] && continue
	out=$("$bin/study" "$log" --against "$candidate" --against-side "$side" --against-show 0 --against-until "${UNTIL:-0}" 2>/dev/null) || continue
	overlap=$(echo "$out" | sed -n 's/^cell overlap *\([0-9]*\)% (\([0-9]*\) of \([0-9]*\)).*/\1 \2 \3/p')
	ink=$(echo "$out" | sed -n 's/^same region disrupted *\([0-9]*\)%.*/\1/p')
	turns=$(echo "$out" | sed -n 's/^turns compared *\([0-9]*\).*/\1/p')
	set -- $overlap
	[ $# -eq 3 ] || continue
	total_shared=$((total_shared + $2))
	total_union=$((total_union + $3))
	total_disrupt=$((total_disrupt + ink * turns))
	total_turns=$((total_turns + turns))
	printf '%-44s %5s%% %5s%% %6s\n' "$(basename "$(dirname "$log")" | cut -c1-8)/$name/p$side" "$1" "$ink" "$turns"
done
printf '%-44s %5s%% %5s%%\n' pooled "$((100 * total_shared / (total_union > 0 ? total_union : 1)))" "$((total_disrupt / (total_turns > 0 ? total_turns : 1)))"
