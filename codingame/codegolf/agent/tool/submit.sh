#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: submit.sh <puzzle/<p>/solution/<lang>/file>"
solution=$(realpath "${1:?$usage}")
relative=${solution#"$root"/}
[[ $relative =~ ^puzzle/([^/]+)/solution/([^/]+)/[^/]+$ ]] || { echo "$usage" >&2; exit 2; }
puzzle=${BASH_REMATCH[1]}
lang=${BASH_REMATCH[2]}
pretty_id=$(tsv_field puzzle "$puzzle" pretty_id)
cg_name=$(tsv_field lang "$lang" cg_name)
cell=$(ensure_status cell "$puzzle" "$lang")

"$root/puzzle/$puzzle/referee/run.sh" "$solution" --validators >/dev/null || {
	echo "submit.sh: refused, local referee fails: run puzzle/$puzzle/referee/run.sh $relative --validators" >&2
	exit 1
}
length=$("$tool_dir/count.sh" "$solution")
ours=$(jq -r '.ours // empty' "$cell")
if [[ -n $ours ]] && ((length >= ${ours%.*})); then
	echo "submit.sh: refused, length $length not below ours $ours" >&2
	exit 1
fi

body=$(mktemp)
trap 'rm -f "$body"' EXIT
jq -n --rawfile code "$solution" --arg language "$cg_name" '{language: $language, code: $code}' >"$body"
submission=$("$tool_dir/cg.sh" POST "/puzzles/$pretty_id/submissions" "$body" | jq -r .submission_id)
event "submit $puzzle $lang length=$length submission=$submission"

for _ in $(seq 60); do
	progress=$("$tool_dir/cg.sh" GET "/puzzles/$pretty_id/progress")
	if [[ $(jq -r .submission_id <<<"$progress") == "$submission" && $(jq -r .in_progress <<<"$progress") == false ]]; then
		break
	fi
	sleep 10
done
sleep 30

row=$(leaderboard_rows "$puzzle" KEYWORD "$nickname" |
	jq -c --arg language "$cg_name" '[.[] | select(.language == $language)] | first')
if [[ $(jq -r '.length // empty' <<<"$row") != "$length" || $(jq -r '.score // 0' <<<"$row") != 100 ]]; then
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
