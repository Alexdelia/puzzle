#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

usage="usage: next.sh <worker-id> <worker-pid> [task-id]; task-id: claim that queued task instead of the best one; exit 0 claimed (prints running file), 1 nothing claimable, 3 rate limited, 4 over weekly pace (STOP set), 5 five-hour window full (prints running task count); when only some kinds run on claude (claude_kinds), a full claude window holds those kinds back instead"
worker=${1:?$usage}
worker_pid=${2:?$usage}
forced_task=${3:-}

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
			elif .kind == "review" or .kind == "wrap" then $context.langs[.lang] == "ready"
			elif .kind == "referee" then $context.langs.python == "ready" and $context.referees[.puzzle] != "blocked"
			elif .kind == "validator" or .kind == "puzzle" then $context.referees[.puzzle] == "done"
			else $context.referees[.puzzle] == "done" and $context.ready_langs == $context.lang_count
			end;
		def far($context): .ours == null or .top == null or .top == 0 or .ours / .top >= $context.far_ratio;
		def plateau_rank($context; $lang; $leads_puzzle):
			if $leads_puzzle or ($context.core_langs | index($lang)) then 0
			elif far($context) and (.plateau_streak // 0) < $context.max_streak then 0
			else .plateau_count // 0 end;
		def kind_rank: {"lang": 0, "referee": 1, "validator": 2, "wrap": 3, "puzzle": 4, "cell": 5, "review": 6}[.kind];
		def core_ladders($context; $live_keys):
			$context.cells | to_entries
			| map(select((.value.top // 0) > 0) | (.key | split("/")) as [$puzzle, $lang]
				| {puzzle: $puzzle, lang: $lang, top: .value.top,
				   reached: (.value.ours != null and .value.ours <= .value.top),
				   live: ("cell/" + .key | IN($live_keys[]))})
			| group_by(.puzzle)
			| map((map(.top) | min) as $shortest_top
				| map(select(.top == $shortest_top or (.lang | IN($context.core_langs[])))) as $core
				| ($core | group_by(.top) | map({top: .[0].top, langs: map(.lang), reached: any(.reached), live: any(.live)})) as $levels
				| ($levels | map(select((.reached | not) and .live)) | first) as $frontier
				| {(.[0].puzzle): {core_langs: ($core | map(.lang)), leader_reached: $levels[0].reached, frontier: $frontier}})
			| add // {};
		def ladder_rank($ladders; $task; $cell):
			$ladders[$task.puzzle] as $ladder
			| if $task.kind != "cell" or $ladder == null then 1
			elif ($task.lang | IN($ladder.core_langs[])) | not then (if $ladder.leader_reached then 1 else 2 end)
			elif $ladder.frontier == null then 1
			elif $task.lang | IN($ladder.frontier.langs[]) then 0
			elif $cell.top > $ladder.frontier.top then "held"
			else 1 end;
		$context_file[0] as $context
		| core_ladders($context; map(.key) + $context.running_keys) as $ladders
		| map(select(ready($context) and ($context.only_kind == "" or .kind == $context.only_kind) and ((.kind | IN($context.held_kinds[])) | not) and ((.key | IN($context.running_keys[])) | not)))
		| map(. as $task
			| ($context.cells[$task.puzzle + "/" + $task.lang] // {}) as $cell
			| ladder_rank($ladders; $task; $cell) as $ladder_rank
			| select($ladder_rank != "held")
			| {id: $task.id, key: [
				($task | kind_rank),
				(if $task.urgent then 0 else 1 end),
				$ladder_rank,
				($cell | plateau_rank($context; $task.lang; $ladder_rank == 0)),
				($context.lang_priority[$task.lang] // 99),
				(if $task.lang == $context.last_lang then 0 else 1 end),
				(if $cell.ours != null and $cell.top != null then $cell.top - $cell.ours else -1000000 end),
				$task.queued_at]})
		| sort_by(.key) | first | .id // empty
	' "$state"/queue/*.json
}

running_claude_tasks() {
	jq -s --arg backend "$backend" '[.[] | select((.backend // $backend) == "claude")] | length' "$state"/running/*.json </dev/null
}

claude_gate() {
	local running_tasks
	running_tasks=$(running_claude_tasks)
	if over_weekly_pace; then
		echo 4
	elif rejected_until >/dev/null; then
		echo 3
	elif five_hour_full "$running_tasks"; then
		echo 5 "$running_tasks"
	fi
}

claim() {
	local queued=("$state"/queue/*.json)
	((${#queued[@]})) || return 1
	local gate running_tasks held_kinds=
	read -r gate running_tasks < <(claude_gate) || true
	if [[ -n $gate ]] && every_task_on_claude; then
		case $gate in
		4)
			touch "$state/STOP"
			event "attention weekly usage $(weekly_pace | tr '\t' ' ') (used elapsed %): over pace, STOP set, restart by hand"
			;;
		5) echo "$running_tasks" ;;
		esac
		return "$gate"
	fi
	[[ -n $gate ]] && held_kinds=$claude_kinds
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
		--argjson running_keys "$(jq -s '[.[].key]' "$state"/running/*.json </dev/null)" \
		--arg only_kind "${CG_ONLY_KIND:-}" --arg held_kinds "$held_kinds" --arg core_langs "$core_langs" --argjson far_ratio "$far_from_top_ratio" --argjson max_streak "$max_plateau_streak" \
		'{langs: $langs[0], referees: $referees[0], cells: $cells[0], last_lang: $last_lang, lang_count: $lang_count, lang_priority: $lang_priority[0],
		  only_kind: $only_kind, held_kinds: ($held_kinds | split(" ") | map(select(. != ""))), core_langs: ($core_langs | split(" ")), far_ratio: $far_ratio, max_streak: $max_streak,
		  ready_langs: ([$langs[0][] | select(. == "ready")] | length),
		  running_keys: $running_keys}' >"$work/context.json"
	if [[ -n $forced_task ]]; then
		id=$([[ -f $state/queue/$forced_task.json ]] && echo "$forced_task" || true)
	else
		id=$(pick "$work/context.json")
	fi
	rm -rf "$work"
	[[ -n $id ]] || return 1
	local running=$state/running/$id.json
	mv "$state/queue/$id.json" "$running"
	local kind puzzle lang status_path task_backend task_model=$model task_effort=$effort
	read -r kind puzzle lang < <(jq -r '[.kind, .puzzle, .lang] | @tsv' "$running")
	status_path=$(ensure_status "$kind" "$puzzle" "$lang")
	task_backend=$(task_backend_of "$kind")
	if [[ $task_backend == opencode ]]; then
		task_model=$(opencode_task_model) task_effort=$opencode_effort
	elif [[ -z ${CG_BEST_MODEL:-} && $kind == cell ]] && (($(jq '.plateau_count // 0' "$status_path") < cell_escalate_after_plateaus)); then
		task_model=$cell_model task_effort=$cell_effort
	fi
	json_update "$running" --arg started_at "$(now)" --arg worker "$worker" --argjson worker_pid "$worker_pid" \
		--arg backend "$task_backend" --arg model "$task_model" --arg effort "$task_effort" --argjson ours_at_claim "$(jq '.ours // null' "$status_path")" \
		'. + {started_at: $started_at, worker: $worker, worker_pid: $worker_pid, backend: $backend, model: $model, effort: $effort, ours_at_claim: $ours_at_claim}'
	json_update "$status_path" --arg id "$id" '.status = "active" | .last_task = $id'
	[[ $kind == cell ]] && echo "$lang" >"$state/last_cell_lang"
	event "claim $id worker=$worker backend=$task_backend model=$task_model effort=$task_effort"
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
