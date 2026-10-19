#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: sync.sh [--if-stale] [puzzle...]"
if [[ ${1:-} == --if-stale ]]; then
	shift
	if [[ -s $state/last_sync ]] && (($(date +%s) - $(date -d "$(<"$state/last_sync")" +%s) < sync_max_age_days * 86400)); then
		echo "sync.sh: last sync $(<"$state/last_sync"), younger than $sync_max_age_days days"
		exit 0
	fi
fi
selected=("$@")
((${#selected[@]})) || mapfile -t selected < <(puzzles)

for puzzle in "${selected[@]}"; do
	[[ -n $(tsv_field puzzle "$puzzle" slug) ]] || { echo "$usage" >&2; exit 2; }
	mine=$(leaderboard_rows "$puzzle" KEYWORD "$nickname")
	table=$state/leaderboard/$puzzle.tsv
	printf 'lang\ttop\ttop_player\ttop_count\tours\trank\tbest_tie_rank\tplayers\tfetched_at\n' >"$table.tmp"
	while read -r lang; do
		cg_name=$(tsv_field lang "$lang" cg_name)
		summary=$(leaderboard_rows "$puzzle" LANGUAGE "$cg_name" | leaderboard_summary)
		if [[ $(jq .ours <<<"$summary") == null ]]; then
			ours=$(jq --arg language "$cg_name" '[.[] | select(.language == $language) | .length] | min' <<<"$mine")
			summary=$(jq -c --argjson ours "$ours" '.ours = $ours' <<<"$summary")
		fi
		jq -r --arg lang "$lang" --arg at "$(now)" \
			'[$lang, .top, .top_player, .top_count, .ours, .rank, .best_tie_rank, .players, $at] | map(. // "-") | @tsv' \
			<<<"$summary" >>"$table.tmp"
		cell=$(ensure_status cell "$puzzle" "$lang")
		previous=$(jq -r .status "$cell")
		json_update "$cell" --argjson summary "$summary" \
			'. + ($summary | {top, top_player, ours, rank, best_tie_rank})
			 | if .best_tie_rank == 1 then .status = "first" else . end'
		if [[ $previous == first && $(jq -r .best_tie_rank "$cell") != 1 ]]; then
			json_update "$cell" '.status = "todo"'
			"$tool_dir/enqueue.sh" cell "$puzzle" "$lang" --reason dropped >/dev/null
			event "dropped $puzzle $lang"
		fi
		sleep 0.2
	done < <(langs)
	mv "$table.tmp" "$table"
	echo "sync.sh: $puzzle"
done
((${#@})) || now >"$state/last_sync"
event "sync ${*:-all}"
