#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

usage="usage: next.sh <worker-id> <worker-pid>; exit 0 claimed (prints running file), 1 nothing claimable, 3 budget exhausted"
worker=${1:?$usage}
worker_pid=${2:?$usage}

status_map() {
	jq -n 'reduce inputs as $entry ({}; . + {(input_filename | split("/") | last | rtrimstr(".json")): $entry.status})' "$@" </dev/null
}

cell_map() {
	jq -n 'reduce inputs as $entry ({}; . + {(input_filename | split("/") | .[-2:] | join("/") | rtrimstr(".json")): $entry})' "$@" </dev/null
}

pick() {
	jq -r -s --argjson context "$1" '
		def ready($context):
			if .kind == "lang" then $context.langs[.lang] != "blocked"
			elif .kind == "referee" then $context.langs.python == "ready" and $context.referees[.puzzle] != "blocked"
			elif .kind == "puzzle" then $context.referees[.puzzle] == "done"
			else $context.referees[.puzzle] == "done" and $context.ready_langs == $context.lang_count
			end;
		def kind_rank: {"lang": 0, "referee": 1, "puzzle": 2, "cell": 3}[.kind];
		map(select(ready($context)))
		| map(. as $task
			| ($context.cells[$task.puzzle + "/" + $task.lang] // {}) as $cell
			| {id: $task.id, key: [
				($task | kind_rank),
				($cell.plateau_count // 0),
				(if $task.lang == $context.last_lang then 0 else 1 end),
				(if $cell.ours != null and $cell.top != null then $cell.top - $cell.ours else -1000000 end),
				$task.queued_at]})
		| sort_by(.key) | first | .id // empty
	' "$state"/queue/*.json
}

claim() {
	local queued=("$state"/queue/*.json)
	((${#queued[@]})) || return 1
	local spent reserved
	spent=$(budget_spent)
	reserved=$(budget_reserved)
	if awk -v spent="$spent" -v reserved="$reserved" -v budget="$task_budget_usd" -v cap="$cost_cap_usd" \
		'BEGIN { exit !(spent + reserved + budget > cap) }'; then
		return 3
	fi
	local langs_json referees_json context id
	langs_json=$(status_map "$state"/lang/*.json)
	referees_json=$(status_map "$state"/referee/*.json)
	context=$(jq -n --argjson langs "$langs_json" --argjson referees "$referees_json" \
		--argjson cells "$(cell_map "$state"/cell/*/*.json)" \
		--arg last_lang "$(cat "$state/last_cell_lang" 2>/dev/null || true)" \
		--argjson lang_count "$(langs | wc -l)" \
		'{langs: $langs, referees: $referees, cells: $cells, last_lang: $last_lang, lang_count: $lang_count,
		  ready_langs: ([$langs[] | select(. == "ready")] | length)}')
	id=$(pick "$context")
	[[ -n $id ]] || return 1
	local running=$state/running/$id.json
	mv "$state/queue/$id.json" "$running"
	json_update "$running" --arg started_at "$(now)" --arg worker "$worker" --argjson worker_pid "$worker_pid" \
		--argjson budget "$task_budget_usd" \
		'. + {started_at: $started_at, worker: $worker, worker_pid: $worker_pid, budget_usd: $budget}'
	local kind puzzle lang status_path
	read -r kind puzzle lang < <(jq -r '[.kind, .puzzle, .lang] | @tsv' "$running")
	status_path=$(ensure_status "$kind" "$puzzle" "$lang")
	json_update "$status_path" --arg id "$id" '.status = "active" | .last_task = $id'
	[[ $kind == cell ]] && echo "$lang" >"$state/last_cell_lang"
	event "claim $id worker=$worker"
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
status=0
with_lock queue claim || status=$?
exit "$status"
