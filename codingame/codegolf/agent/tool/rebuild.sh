#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

last_outcome() {
	awk -F'\t' -v kind="$1" -v puzzle="$2" -v lang="$3" \
		'NR > 1 && $3 == kind && $4 == puzzle && $5 == lang { outcome = $12; note = $13 } END { print outcome "\t" note }' \
		"$state/time.log"
}

tracked() {
	[[ -n $(git -C "$root" ls-files -- "$1") ]]
}

for lang in $(langs); do
	file=$(ensure_status lang - "$lang")
	IFS=$'\t' read -r outcome note < <(last_outcome lang - "$lang")
	if tracked "agent/tool/lang/$lang/build.sh"; then
		status=ready
	elif [[ $outcome == blocked ]]; then
		status=blocked
	else
		status=todo
	fi
	json_update "$file" --arg status "$status" --arg note "$note" '.status = $status | .note = $note'
	IFS=$'\t' read -r outcome note < <(last_outcome wrap - "$lang")
	if [[ -n $outcome ]] || tracked "agent/wrap/$lang/available.tsv"; then
		file=$(ensure_status wrap - "$lang")
		if tracked "agent/wrap/$lang/available.tsv"; then
			status=done
		elif [[ $outcome == blocked ]]; then
			status=blocked
		else
			status=todo
		fi
		json_update "$file" --arg status "$status" '.status = $status'
	fi
	IFS=$'\t' read -r outcome note < <(last_outcome review - "$lang")
	if [[ -n $outcome ]]; then
		file=$(ensure_status review - "$lang")
		json_update "$file" --arg status "$([[ $outcome == done ]] && echo done || echo todo)" '.status = $status'
	fi
done

for puzzle in $(puzzles); do
	file=$(ensure_status referee "$puzzle" -)
	IFS=$'\t' read -r outcome note < <(last_outcome referee "$puzzle" -)
	if tracked "puzzle/$puzzle/referee/bench.txt" && grep -q '^suite .* pass$' "$root/puzzle/$puzzle/referee/bench.txt"; then
		status=done
	elif [[ $outcome == blocked ]]; then
		status=blocked
	else
		status=todo
	fi
	json_update "$file" --arg status "$status" --arg note "$note" '.status = $status | .note = $note'
	IFS=$'\t' read -r outcome note < <(last_outcome validator "$puzzle" -)
	if [[ -n $outcome ]] || tracked "puzzle/$puzzle/referee/hidden/facts.md"; then
		file=$(ensure_status validator "$puzzle" -)
		if tracked "puzzle/$puzzle/referee/hidden/facts.md"; then
			status=done
		elif [[ $outcome == blocked ]]; then
			status=blocked
		else
			status=todo
		fi
		json_update "$file" --arg status "$status" --arg note "$note" '.status = $status | .note = $note'
	fi
	IFS=$'\t' read -r outcome note < <(last_outcome puzzle "$puzzle" -)
	if [[ -n $outcome ]]; then
		file=$(ensure_status puzzle "$puzzle" -)
		json_update "$file" --arg status "$([[ $outcome == done ]] && echo done || echo todo)" '.status = $status'
	fi
done

[[ -n $(ls "$state"/leaderboard/*.tsv 2>/dev/null) ]] || "$tool_dir/sync.sh"

awk -F'\t' 'NR > 1 && $3 == "cell" { key = $4 "/" $5; attempts[key]++; last[key] = $12
		if ($12 != "first" && $12 != "continue") plateaus[key]++ }
	END { for (key in attempts) print key "\t" attempts[key] "\t" plateaus[key] + 0 "\t" last[key] }' "$state/time.log" |
	while IFS=$'\t' read -r key attempts plateaus last; do
		file=$state/cell/$key.json
		[[ -f $file ]] || continue
		json_update "$file" --argjson attempts "$attempts" --argjson plateaus "$plateaus" --arg last "$last" \
			'.attempts = $attempts | .plateau_count = $plateaus
			 | if .status != "first" and $last != "first" and $last != "continue" then .status = "plateau" else . end'
	done

event "rebuild status from time.log, git and leaderboard"
echo "rebuild.sh: status rebuilt"
