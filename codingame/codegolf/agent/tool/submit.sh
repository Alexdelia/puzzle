#!/usr/bin/env bash
set -Eeuo pipefail
source "$(dirname "$0")/lib.sh"
trap 'echo "submit.sh: failed at line $LINENO (exit $?): $BASH_COMMAND" >&2' ERR

usage="usage: submit.sh [--no-wait | --judge <row-json>] [--candidate <file>] <puzzle/<p>/solution/<lang>/main.<ext>> | submit.sh --probe <puzzle/<p>/probe/<lang>/<name>.<ext>>; --no-wait: gates and POST only, prints the submission id; --judge: no gate, no POST, judges the code against our leaderboard row read by the caller; --candidate: the code comes from <file>, copied into main.<ext> only on a confirmed gain"
probe=
mode=full
candidate=
while (($# > 1)); do
	case $1 in
	--probe) probe=1 && shift ;;
	--no-wait) mode=no-wait && shift ;;
	--judge) mode=judge judged_row=${2:?$usage} && shift 2 ;;
	--candidate) candidate=$(realpath "${2:?$usage}") && shift 2 ;;
	*) echo "$usage" >&2 && exit 2 ;;
	esac
done
solution=$(realpath -m "${1:?$usage}")
code=${candidate:-$solution}
relative=${solution#"$root"/}
[[ $relative =~ ^puzzle/([^/]+)/(solution|probe)/([^/]+)/[^/]+$ ]] || { echo "$usage" >&2; exit 2; }
puzzle=${BASH_REMATCH[1]}
area=${BASH_REMATCH[2]}
lang=${BASH_REMATCH[3]}
if [[ -n $probe && $area != probe ]] || [[ -z $probe && $area != solution ]]; then
	echo "submit.sh: probes live in puzzle/$puzzle/probe/$lang/<name>.<ext> and go with --probe; solutions are puzzle/$puzzle/solution/$lang/main.<ext> without it" >&2
	echo "$usage" >&2
	exit 2
fi
pretty_id=$(tsv_field puzzle "$puzzle" pretty_id)
cg_name=$(tsv_field lang "$lang" cg_name)
cell=$(ensure_status cell "$puzzle" "$lang")
probe_log=$root/puzzle/$puzzle/probe/probes.tsv
probe_pending="pending, submission"

started=$EPOCHSECONDS
marks=()
mark() {
	marks+=("$1 $((EPOCHSECONDS - started))s")
	started=$EPOCHSECONDS
}
report_time() {
	((${#marks[@]})) && echo "submit.sh: time ${marks[*]}" >&2 || true
}
length=$("$tool_dir/count.sh" "$code")
ours=$(jq -r '.ours // empty' "$cell")
fetch_own_row() {
	own_rows=$(leaderboard_rows "$puzzle" KEYWORD "$nickname")
	row=$(jq -c --arg language "$cg_name" '[.[] | select(.language == $language)] | first' <<<"$own_rows")
}

log_pending_probe() {
	[[ -n $probe ]] || return 0
	local question=${CG_PROBE_QUESTION:-no question given: ${code#"$root"/}}
	mkdir -p "$(dirname "$probe_log")"
	with_lock oracle bash -c 'log=$1 && shift && printf "%s\t%s\t%s\t%s\t%s\t%s\n" "$@" >>"$log"' _ "$probe_log" \
		"$(now)" "${CG_TASK_ID:--}" "$lang" "$length" "${question//[$'\t\n']/ }" "$probe_pending $submission"
}

resolve_probe() {
	[[ -n $probe && -n ${submission:-} ]] || return 0
	local answer=${1//[$'\t\n']/ }
	with_lock oracle bash -c 'awk -F"\t" -v OFS="\t" -v pending="$2" -v answer="$3" "\$6 == pending { \$6 = answer } 1" "$1" >"$1.tmp" && mv "$1.tmp" "$1"' \
		_ "$probe_log" "$probe_pending $submission" "$answer"
}

validator_report=
fetch_validator_report() {
	local attempt
	for ((attempt = 1; attempt <= report_wait_attempts; attempt++)); do
		validator_report=$("$tool_dir/cg-report.sh" "$submission" 2>/dev/null) || break
		jq -e .completed <<<"$validator_report" >/dev/null && break
		sleep 10
	done
	jq -e .completed <<<"$validator_report" >/dev/null 2>&1 && printf '%s\n' "$validator_report" >"$state/report/$submission.json"
	return 0
}

report_completed() {
	jq -e "${1:-true} and .completed" <<<"$validator_report" >/dev/null 2>&1
}

report_line() {
	if report_completed; then
		jq -r '([.validators[].weight] | unique | length > 1) as $weighted
			| "report \(.score)%: " + ([.validators | to_entries[] | "\(.key + 1) \(.value.name)\(if .value.method then " [\(.value.method)]" else "" end) \(if .value.success then "pass" else "fail" end)"
				+ (if $weighted then " (weight \(.value.weight))" else "" end)] | join(", "))' <<<"$validator_report"
	else
		local reason
		reason=$(jq -r 'if .completed == false then "not completed" else .error // empty end' <<<"$validator_report" 2>/dev/null || true)
		echo "report unavailable (${reason:-no report})"
	fi
}

submit_and_wait() {
	recent_submits=$(awk -F'\t' -v since="$(date -u -d '5 hours ago' +%Y-%m-%dT%H:%M:%SZ)" -v path="/puzzles/$pretty_id/submissions" \
		'$1 >= since && $3 == "POST" && $4 == path && $5 == 201' "$state/cg.log" 2>/dev/null | wc -l)
	if ((recent_submits >= submit_cap_5h)); then
		echo "submit.sh: refused, $recent_submits agent submits on $puzzle in the last 5 h (cap $submit_cap_5h per puzzle, rest kept for the user): note it in the journal and move on" >&2
		exit 1
	fi
	hidden_complete=$root/puzzle/$puzzle/referee/hidden/complete
	local_sets=(--validators)
	[[ -e $hidden_complete ]] && local_sets=(--hidden)
	probe_cap=$probe_submits_per_task
	[[ ${CG_TASK_ID:-} == validator.* ]] && probe_cap=$validator_submits_per_task
	if [[ -n $probe ]]; then
		task_submits=$(awk -F'\t' -v id="${CG_TASK_ID:--}" '$2 == id && $3 == "POST" && $4 ~ /\/submissions$/ && $5 == 201' "$state/cg.log" 2>/dev/null | wc -l)
		if ((task_submits >= probe_cap)); then
			echo "submit.sh: refused, this task already used $task_submits submits (probe cap $probe_cap)" >&2
			exit 1
		fi
	elif ! "$root/puzzle/$puzzle/referee/run.sh" "$code" --lang "$lang" "${local_sets[@]}" >/dev/null; then
		echo "submit.sh: refused, local referee fails: run puzzle/$puzzle/referee/run.sh ${code#"$root"/} --lang $lang ${local_sets[*]} (hidden-validator exploit: use --probe)" >&2
		exit 1
	elif [[ -n $ours ]] && ((length >= ${ours%.*})); then
		echo "submit.sh: refused, length $length not below ours $ours" >&2
		exit 1
	fi
	mark referee

	body=$(mktemp)
	trap 'rm -f "$body"; report_time' EXIT
	jq -n --rawfile code "$code" --arg language "$cg_name" '{language: $language, code: $code}' >"$body"
	if ! response=$("$tool_dir/cg.sh" POST "/puzzles/$pretty_id/submissions" "$body"); then
		echo "submit.sh: POST failed for length $length: $(head -c 300 <<<"$response" | tr '\n' ' ')" >&2
		exit 1
	fi
	submission=$(jq -r '.submission_id // empty' <<<"$response" 2>/dev/null || true)
	if [[ ! $submission =~ ^[0-9]+$ ]]; then
		echo "submit.sh: POST failed for length $length, no submission id: $(head -c 300 <<<"$response" | tr '\n' ' ')" >&2
		exit 1
	fi
	event "${probe:+probe }submit $puzzle $lang length=$length submission=$submission"
	log_pending_probe
	if [[ $mode == no-wait ]]; then
		echo "$submission"
		exit 0
	fi

	for _ in $(seq 60); do
		progress=$("$tool_dir/cg.sh" GET "/puzzles/$pretty_id/progress")
		current=$(jq -r '.submission_id // 0' <<<"$progress")
		[[ $current == "$submission" && $(jq -r .in_progress <<<"$progress") == false ]] && break
		((current > submission)) && break
		sleep 10
	done
	mark cg
	fetch_validator_report
	mark report

	fetch_own_row
	for _ in $(seq 21); do
		(($(jq -r '.length // 1e9 | floor' <<<"$row") <= length)) && break
		report_completed '.score < 100' && break
		sleep 10
		fetch_own_row
	done
	store_own_rows "$puzzle" "$own_rows" "$lang"
	mark row
}

if [[ $mode == judge ]]; then
	row=$judged_row
else
	submit_and_wait
fi
if [[ -n $probe ]] && { [[ $(jq -r '.length // empty' <<<"$row") != "$length" ]] || ! jq -e '.score == 100' <<<"$row" >/dev/null; }; then
	resolve_probe "$(report_line), own row $row"
	echo "submit.sh: probe result, $(report_line)"
	echo "submit.sh: own row $row (CG keeps the best submission per language: a lower score than the row shown stays hidden)"
	if report_completed '.score == 100' && [[ -n $ours ]] && ((length < ${ours%.*})); then
		echo "submit.sh: CG scored this probe 100% but our row did not move (lost leaderboard update): copy it to the solution file and submit it again"
	fi
	exit 3
fi
row_length=$(jq -r '.length // empty | floor' <<<"$row")
if [[ -n $row_length ]] && ((row_length < length)) && jq -e '.score == 100' <<<"$row" >/dev/null; then
	event "superseded $puzzle $lang length=$length row=$row"
	echo "submit.sh: superseded, CG already holds a shorter row $row_length at 100% (concurrent submit): copy it from the leaderboard or git log before going on" >&2
	exit 1
fi
if [[ -n ${CG_WRAP_SOURCE:-} ]] && { [[ $(jq -r '.length // empty' <<<"$row") != "$length" ]] || ! jq -e '.score == 100' <<<"$row" >/dev/null; }; then
	hash=$(sha256sum <"$code" | cut -c1-64)
	round=$(wrap_lottery_round "$hash" "$puzzle" "$lang" "$length" "$CG_WRAP_SOURCE" "$row")
	if ((round)); then
		event "wrap lottery $puzzle $lang length=$length round=$round/$wrap_lottery_rounds source=$CG_WRAP_SOURCE row=$row"
		echo "submit.sh: lottery round $round of $wrap_lottery_rounds lost for generated wrapper length $length, next round in a later wrap run: $row" >&2
		exit 1
	fi
	mkdir -p "$(dirname "$wrap_rejected")"
	printf '%s\t%s\t%s\t%s\t%s\n' "$hash" "$puzzle" "$lang" "$length" "$CG_WRAP_SOURCE" >>"$wrap_rejected"
	event "attention wrap rejected $puzzle $lang length=$length source=$CG_WRAP_SOURCE row=$row"
	echo "submit.sh: CG did not confirm generated wrapper length $length: $row" >&2
	exit 1
fi
if [[ $(jq -r '.length // empty' <<<"$row") != "$length" ]] || ! jq -e '.score == 100' <<<"$row" >/dev/null; then
	if report_completed '.score == 100'; then
		event "unconfirmed $puzzle $lang length=$length: report 100%, row $row (lost leaderboard update)"
		echo "submit.sh: CG scored length $length at 100% but our row did not move (lost leaderboard update): submit the same file again" >&2
		exit 1
	fi
	json_update "$(ensure_status referee "$puzzle" -)" '.status = "suspect"'
	"$tool_dir/enqueue.sh" referee "$puzzle" - --reason "CG rejected $relative length $length: row $row" >/dev/null
	event "rejected $puzzle $lang length=$length row=$row $(report_line)"
	echo "submit.sh: CG did not confirm length $length at 100%: $(report_line); own row $row" >&2
	exit 1
fi

summary=$(leaderboard_rows "$puzzle" LANGUAGE "$cg_name" | leaderboard_summary | jq -c --argjson length "$length" '.ours = $length')
store_language_summary "$puzzle" "$lang" "$summary"
store_table_row "$puzzle" "$lang" "$summary"
[[ -z ${CG_WRAP_SOURCE:-} ]] || json_update "$cell" --arg wrap_source "$CG_WRAP_SOURCE" '.wrap_source = $wrap_source'
rank=$(jq -r '.rank // "1000+"' "$cell")
best_tie_rank=$(jq -r '.best_tie_rank // empty' "$cell")
message="feat(golf): solution $puzzle $lang $length #$rank"
if [[ -n $best_tie_rank && $best_tie_rank != "$rank" ]]; then
	message+=" (tie #$best_tie_rank)"
fi
main=$root/puzzle/$puzzle/solution/$lang/main.$(tsv_field lang "$lang" extension)
if [[ $code != "$main" ]]; then
	mkdir -p "$(dirname "$main")"
	cp "$code" "$main"
fi
resolve_probe "yes: 100% at length $length, committed"
commit_scope "$message" "puzzle/$puzzle/solution/$lang"
mark commit
event "gain $puzzle $lang length=$length previous=${ours%.*} rank=$rank best_tie_rank=${best_tie_rank:--}${CG_WRAP_SOURCE:+ source=${CG_WRAP_SOURCE%%:*}}"
! wrap_cores | grep -q -x -F "$lang" || wrap_enqueue "$puzzle" "core $lang $length"
echo "submit.sh: $message"
