#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

usage="usage: next.sh <worker-id> <worker-pid>; exit 0 claimed (prints running file), 1 nothing claimable, 3 rate limited, 4 over weekly pace (STOP set), 5 five-hour window full (prints running task count)"
worker=${1:?$usage}
worker_pid=${2:?$usage}

status_map() {
	jq -n 'reduce inputs as $entry ({}; . + {(input_filename | split("/") | last | rtrimstr(".json")): $entry.status})' "$@" </dev/null
}

cell_map() {
	jq -n 'reduce inputs as $entry ({}; . + {(input_filename | split("/") | .[-2:] | join("/") | rtrimstr(".json")): $entry})' "$@" </dev/null
}

lang_priority() {
	awk -F'\t' 'NR == 1 { for (i = 1; i <= NF; i++) column[$i] = i; next } { print $1 "\t" $column["priority"] }' "$root/data/lang.tsv" |
		jq -R -n '[inputs | split("\t") | {(.[0]): (.[1] | tonumber)}] | add'
}

pick() {
	jq -r -s --slurpfile context_file "$1" '
		def ready($context):
			if .kind == "lang" then $context.langs[.lang] != "blocked"
			elif .kind == "referee" then $context.langs.python == "ready" and $context.referees[.puzzle] != "blocked"
			elif .kind == "validator" or .kind == "puzzle" then $context.referees[.puzzle] == "done"
			else $context.referees[.puzzle] == "done" and $context.ready_langs == $context.lang_count
			end;
		def kind_rank: {"lang": 0, "referee": 1, "validator": 2, "puzzle": 3, "cell": 4}[.kind];
		$context_file[0] as $context
		| map(select(ready($context)))
		| map(. as $task
			| ($context.cells[$task.puzzle + "/" + $task.lang] // {}) as $cell
			| {id: $task.id, key: [
				($task | kind_rank),
				($cell.plateau_count // 0),
				($context.lang_priority[$task.lang] // 99),
				(if $task.lang == $context.last_lang then 0 else 1 end),
				(if $cell.ours != null and $cell.top != null then $cell.top - $cell.ours else -1000000 end),
				$task.queued_at]})
		| sort_by(.key) | first | .id // empty
	' "$state"/queue/*.json
}

claim() {
	local queued=("$state"/queue/*.json)
	((${#queued[@]})) || return 1
	if over_weekly_pace; then
		touch "$state/STOP"
		event "attention weekly usage $(weekly_pace | tr '\t' ' ') (used elapsed %): over pace, STOP set, restart by hand"
		return 4
	fi
	if rejected_until >/dev/null; then
		return 3
	fi
	local running_tasks=("$state"/running/*.json)
	if five_hour_full "${#running_tasks[@]}"; then
		echo "${#running_tasks[@]}"
		return 5
	fi
	local work id
	work=$(mktemp -d)
	status_map "$state"/lang/*.json >"$work/langs.json"
	status_map "$state"/referee/*.json >"$work/referees.json"
	cell_map "$state"/cell/*/*.json >"$work/cells.json"
	lang_priority >"$work/lang_priority.json"
	jq -n --slurpfile langs "$work/langs.json" --slurpfile referees "$work/referees.json" \
		--slurpfile cells "$work/cells.json" --slurpfile lang_priority "$work/lang_priority.json" \
		--arg last_lang "$(cat "$state/last_cell_lang" 2>/dev/null || true)" \
		--argjson lang_count "$(langs | wc -l)" \
		'{langs: $langs[0], referees: $referees[0], cells: $cells[0], last_lang: $last_lang, lang_count: $lang_count, lang_priority: $lang_priority[0],
		  ready_langs: ([$langs[0][] | select(. == "ready")] | length)}' >"$work/context.json"
	id=$(pick "$work/context.json")
	rm -rf "$work"
	[[ -n $id ]] || return 1
	local running=$state/running/$id.json
	mv "$state/queue/$id.json" "$running"
	local kind puzzle lang status_path task_model=$model task_effort=$effort
	read -r kind puzzle lang < <(jq -r '[.kind, .puzzle, .lang] | @tsv' "$running")
	status_path=$(ensure_status "$kind" "$puzzle" "$lang")
	if [[ -z ${CG_BEST_MODEL:-} && $kind == cell ]] && (($(jq '.plateau_count // 0' "$status_path") < cell_escalate_after_plateaus)); then
		task_model=$cell_model task_effort=$cell_effort
	fi
	json_update "$running" --arg started_at "$(now)" --arg worker "$worker" --argjson worker_pid "$worker_pid" \
		--arg model "$task_model" --arg effort "$task_effort" \
		'. + {started_at: $started_at, worker: $worker, worker_pid: $worker_pid, model: $model, effort: $effort}'
	json_update "$status_path" --arg id "$id" '.status = "active" | .last_task = $id'
	[[ $kind == cell ]] && echo "$lang" >"$state/last_cell_lang"
	event "claim $id worker=$worker model=$task_model effort=$task_effort"
	echo "$running"
}

missing_referees() {
	jq -r 'select(.kind == "puzzle" or .kind == "cell") | .puzzle' "$state"/queue/*.json 2>/dev/null | sort -u |
		while read -r puzzle; do
			local status
			status=$(jq -r .status "$(ensure_status referee "$puzzle" -)")
			if [[ $status == todo ]] && ! grep -q -F "\"key\": \"referee/$puzzle/-\"" "$state"/queue/*.json "$state"/running/*.json 2>/dev/null; then
				"$tool_dir/enqueue.sh" referee "$puzzle" - --reason "needed by queued tasks" >/dev/null
			fi
		done
}

missing_referees
refresh_usage
status=0
with_lock queue claim || status=$?
exit "$status"
