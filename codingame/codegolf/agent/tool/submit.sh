#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: submit.sh [--probe] <puzzle/<p>/solution/<lang>/file>"
probe=
if [[ ${1:-} == --probe ]]; then
	probe=1
	shift
fi
solution=$(realpath "${1:?$usage}")
relative=${solution#"$root"/}
[[ $relative =~ ^puzzle/([^/]+)/solution/([^/]+)/[^/]+$ ]] || { echo "$usage" >&2; exit 2; }
puzzle=${BASH_REMATCH[1]}
lang=${BASH_REMATCH[2]}
pretty_id=$(tsv_field puzzle "$puzzle" pretty_id)
cg_name=$(tsv_field lang "$lang" cg_name)
cell=$(ensure_status cell "$puzzle" "$lang")

length=$("$tool_dir/count.sh" "$solution")
ours=$(jq -r '.ours // empty' "$cell")
if [[ -n $probe ]]; then
	task_submits=$(awk -F'\t' -v id="${CG_TASK_ID:--}" '$2 == id && $3 == "POST" && $4 ~ /\/submissions$/' "$state/cg.log" 2>/dev/null | wc -l)
	if ((task_submits >= probe_submits_per_task)); then
		echo "submit.sh: refused, this task already used $task_submits submits (probe cap $probe_submits_per_task)" >&2
		exit 1
	fi
elif ! "$root/puzzle/$puzzle/referee/run.sh" "$solution" --validators >/dev/null; then
	echo "submit.sh: refused, local referee fails: run puzzle/$puzzle/referee/run.sh $relative --validators (hidden-validator exploit: use --probe)" >&2
	exit 1
elif [[ -n $ours ]] && ((length >= ${ours%.*})); then
	echo "submit.sh: refused, length $length not below ours $ours" >&2
	exit 1
fi

body=$(mktemp)
trap 'rm -f "$body"' EXIT
jq -n --rawfile code "$solution" --arg language "$cg_name" '{language: $language, code: $code}' >"$body"
submission=$("$tool_dir/cg.sh" POST "/puzzles/$pretty_id/submissions" "$body" | jq -r .submission_id)
event "${probe:+probe }submit $puzzle $lang length=$length submission=$submission"

for _ in $(seq 60); do
	progress=$("$tool_dir/cg.sh" GET "/puzzles/$pretty_id/progress")
	if [[ $(jq -r .submission_id <<<"$progress") == "$submission" && $(jq -r .in_progress <<<"$progress") == false ]]; then
		break
	fi
	sleep 10
done
sleep 30

own_row() {
	leaderboard_rows "$puzzle" KEYWORD "$nickname" |
		jq -c --arg language "$cg_name" '[.[] | select(.language == $language)] | first'
}
row=$(own_row)
for _ in $(seq 6); do
	[[ $(jq -r '.length // empty' <<<"$row") == "$length" ]] && break
	sleep 30
	row=$(own_row)
done
if [[ -n $probe ]] && { [[ $(jq -r '.length // empty' <<<"$row") != "$length" ]] || ! jq -e '.score == 100' <<<"$row" >/dev/null; }; then
	echo "submit.sh: probe result, progress $(jq -c 'del(.limits)' <<<"$progress"), own row $row"
	echo "submit.sh: CG keeps the best submission per language: a lower score than the row shown stays hidden"
	exit 3
fi
if [[ $(jq -r '.length // empty' <<<"$row") != "$length" ]] || ! jq -e '.score == 100' <<<"$row" >/dev/null; then
	json_update "$(ensure_status referee "$puzzle" -)" '.status = "suspect"'
	"$tool_dir/enqueue.sh" referee "$puzzle" - --reason "CG rejected $relative length $length: row $row" >/dev/null
	event "rejected $puzzle $lang length=$length row=$row"
	echo "submit.sh: CG did not confirm length $length at 100%: $row" >&2
	exit 1
fi

summary=$(leaderboard_rows "$puzzle" LANGUAGE "$cg_name" | leaderboard_summary)
json_update "$cell" --argjson summary "$summary" --argjson length "$length" \
	'. + ($summary | {top, top_player, rank, best_tie_rank}) | .ours = $length
	 | if .best_tie_rank == 1 then .status = "first" else . end'
rank=$(jq -r '.rank // "1000+"' "$cell")
best_tie_rank=$(jq -r '.best_tie_rank // empty' "$cell")
message="feat(golf): solution $puzzle $lang $length #$rank"
if [[ -n $best_tie_rank && $best_tie_rank != "$rank" ]]; then
	message+=" (tie #$best_tie_rank)"
fi
commit_scope "$message" "puzzle/$puzzle/solution/$lang"
event "gain $puzzle $lang length=$length rank=$rank best_tie_rank=${best_tie_rank:--}"
echo "submit.sh: $message"
