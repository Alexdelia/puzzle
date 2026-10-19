#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh
shopt -s nullglob

count_status() {
	local dir=$1
	local files=("$dir"/*.json)
	((${#files[@]})) || { echo "none"; return; }
	jq -r .status "${files[@]}" | sort | uniq -c | awk '{ printf "%s %s  ", $2, $1 } END { print "" }'
}

echo "== env"
echo "langs:    $(count_status "$state/lang")"
echo "referees: $(count_status "$state/referee")"
echo "puzzles:  $(count_status "$state/puzzle")"
blocked=$(grep -l '"status": "blocked"' "$state"/lang/*.json "$state"/referee/*.json 2>/dev/null | xargs -r -n1 basename | sed 's/\.json$//' | tr '\n' ' ' || true)
[[ -z $blocked ]] || echo "blocked:  $blocked"

cells=("$state"/cell/*/*.json)
if ((${#cells[@]})); then
	echo
	echo "== cells first/total per lang"
	jq -r '[input_filename, .status] | @tsv' "${cells[@]}" | sed 's|.*/cell/[^/]*/||; s|\.json||' |
		awk -F'\t' '{ total[$1]++; if ($2 == "first") first[$1]++ } END { for (lang in total) printf "%-12s %3d/%d\n", lang, first[lang], total[lang] }' |
		sort | column -c 120
	echo
	echo "== cells first/total per puzzle"
	jq -r '[input_filename, .status] | @tsv' "${cells[@]}" | sed 's|.*/cell/||; s|/[^/]*\.json||' |
		awk -F'\t' '{ total[$1]++; if ($2 == "first") first[$1]++ } END { for (p in total) printf "%-34s %3d/%d\n", p, first[p], total[p] }' |
		sort | column -c 120
	echo
	echo "== smallest gaps not first (ours - top)"
	jq -r 'select(.status != "first" and .ours != null and .top != null) | [(.ours - .top), input_filename, .ours, .top] | @tsv' "${cells[@]}" |
		sed 's|[^\t]*/cell/||; s|\.json||' | sort -n | head -10 | awk -F'\t' '{ printf "  +%-4s %-45s ours %s top %s\n", $1, $2, $3, $4 }'
fi

echo
echo "== work"
queued=("$state"/queue/*.json)
echo "queued: ${#queued[@]}"
for running in "$state"/running/*.json; do
	jq -r '"running: \(.id) worker \(.worker) since \(.started_at)"' "$running"
done
[[ -e $state/STOP ]] && echo "STOP set: workers stop after their current task"

echo
echo "== time and cost (from agent/state/time.log)"
awk -F'\t' 'NR > 1 { wall[$3] += $6; cpu[$3] += $7 + $8; cost[$3] += $11; tasks[$3]++ }
	END { for (k in tasks) printf "  %-8s tasks %4d  wall %7.1f h  cpu %7.1f h  cost $%8.2f\n", k, tasks[k], wall[k] / 3600, cpu[k] / 3600, cost[k] }' "$state/time.log"
echo "  last ${cost_window_hours}h cost: \$$(budget_spent) of \$$cost_cap_usd cap"

reviews=$(grep -P '\treview ' "$state/events.log" 2>/dev/null | tail -5 || true)
attention=$(grep -P '\t(attention|rejected) ' "$state/events.log" 2>/dev/null | tail -5 || true)
if [[ -n $reviews$attention ]]; then
	echo
	echo "== needs you"
	[[ -z $attention ]] || echo "$attention"
	[[ -z $reviews ]] || echo "$reviews"
	git -C "$root" status --short -- . | grep -v -E ' agent/state/|^\?\? agent/state/' | head -20 || true
fi
