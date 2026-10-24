#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: worker.sh <worker-id> [--once | --task <task-id>]; --task: run that queued task once, whatever its priority"
worker=${1:?$usage}
once=
task=
case ${2:-} in
--once) once=1 ;;
--task) task=${3:?$usage} once=1 ;;
"") ;;
*) echo "$usage" >&2 && exit 2 ;;
esac
log() {
	echo "$(now) worker $worker: $*"
}

run_agent() {
	local resume_session=${1:-} agent
	case $task_backend in
	opencode)
		agent=(opencode run --model "$task_model" --variant "$task_effort" --format json --auto)
		if [[ -n $resume_session ]]; then
			agent+=(--session "$resume_session" "$opencode_length_resume_message")
		else
			agent+=(--title "$id" "$(<"$output.prompt")")
		fi
		;;
	*)
		agent=(claude -p "$(<"$output.prompt")" --model "$task_model" --effort "$task_effort"
			--output-format stream-json --verbose --permission-mode auto --max-budget-usd "$task_budget_usd")
		;;
	esac
	local scope=codegolf-task-$id.scope status
	CG_TASK_ID=$id OPENCODE_ENABLE_EXA=1 OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX=$opencode_output_token_max \
		command time -a -f '%e %U %S' -o "$output.time" \
		timeout --kill-after=60 "$((deadline - EPOCHSECONDS))" \
		systemd-run --user --scope --quiet --unit="$scope" --slice=codegolf.slice -p MemoryMax="$task_memory_max" \
		direnv exec "$root" "$tool_dir/sandbox.sh" "${agent[@]}" >>"$output.jsonl" 2>>"$output.err" </dev/null
	status=$?
	if systemctl --user is-active --quiet "$scope"; then
		log "stopping leftover processes of $id: $(systemctl --user show -p TasksCurrent --value "$scope") tasks"
		systemctl --user stop "$scope"
	fi
	return "$status"
}

start_fresh() {
	rm -f "$output.jsonl" "$output.err" "$output.time"
}

sum_time_segments() {
	[[ -s $output.time ]] || return 0
	awk '{ wall += $1; user += $2; sys += $3 } END { printf "%.2f %.2f %.2f\n", wall, user, sys }' "$output.time" >"$output.time.sum"
	mv "$output.time.sum" "$output.time"
}

cd "$root"
worker_pid=$BASHPID
waiting=
while [[ ! -e $state/STOP ]]; do
	running=$("$tool_dir/next.sh" "$worker" "$worker_pid" $task)
	case $? in
	0) ;;
	1)
		[[ -n $once || -n ${CG_ONLY_KIND:-} ]] && break
		sleep 60
		continue
		;;
	3)
		resume_at=$(rejected_until || date +%s)
		log "rate limited until $(date -d "@$resume_at" +%FT%T)"
		sleep $((resume_at - $(date +%s) + 30))
		continue
		;;
	4)
		log "over weekly pace, STOP set"
		break
		;;
	5)
		if ((running > 0)); then
			[[ -n $waiting ]] || log "five-hour window $(five_hour_used)% full, waiting for $running running tasks"
			waiting=1
			sleep 60
		else
			resume_at=$(five_hour_resets_at)
			log "five-hour window $(five_hour_used)% full, sleeping until $(date -d "@$resume_at" +%FT%T)"
			sleep $((resume_at - $(date +%s) + 30))
		fi
		continue
		;;
	*)
		log "next.sh failed"
		sleep 60
		continue
		;;
	esac
	waiting=
	read -r id task_backend task_model task_effort < <(jq -r --arg backend "$backend" '[.id, .backend // $backend, .model, .effort] | @tsv' "$running")
	if [[ ! $id =~ ^[A-Za-z0-9+#-][A-Za-z0-9._+#-]*$ ]]; then
		touch "$state/STOP"
		event "attention worker $worker: task id '$id' in $running is no plain file name, STOP set"
		log "bad task id $id, STOP set"
		break
	fi
	log "start $id backend=$task_backend model=$task_model effort=$task_effort"
	output=$state/worker/$id
	"$tool_dir/prompt.sh" "$running" >"$output.prompt"
	deadline=$((EPOCHSECONDS + task_wall_seconds))
	start_fresh
	run_agent
	code=$?
	if [[ $task_backend == opencode && $task_model != "$opencode_free_model" ]] && credit=$(opencode_out_of_credit "$output.jsonl"); then
		touch "$opencode_fallback_file"
		event "opencode $task_model out of credit ($credit): $id reruns on $opencode_free_model, every later task too until ./start.sh"
		log "out of credit on $task_model, rerun on $opencode_free_model"
		task_model=$opencode_free_model
		json_update "$running" --arg model "$task_model" '.model = $model'
		start_fresh
		run_agent
		code=$?
	fi
	resumes=0
	while [[ $task_backend == opencode ]] && ((resumes < opencode_length_resumes && deadline - EPOCHSECONDS > 60)) &&
		opencode_cut_by_length "$output.jsonl" && jq -e '.outcome == null' "$running" >/dev/null; do
		resumes=$((resumes + 1))
		session=$(opencode_session "$output.jsonl")
		event "opencode $id cut by output cap without report: resume $resumes/$opencode_length_resumes of session $session"
		log "cut by output cap, resume $resumes of $session"
		run_agent "$session"
		code=$?
	done
	sum_time_segments
	if [[ $task_backend == opencode ]]; then
		opencode_result "$output.jsonl" >"$output.json"
		if fatal=$(opencode_fatal_error "$output.jsonl"); then
			touch "$state/STOP"
			event "attention opencode $task_model failed ($fatal): STOP set, fix the provider account and restart by hand"
			log "provider error $fatal, STOP set"
		fi
	else
		jq -c 'select(.type == "result")' "$output.jsonl" | tail -n 1 >"$output.json"
	fi
	record_usage "$output.jsonl"
	"$tool_dir/finish.sh" "$running" "$output.json" "$output.time" "$code"
	log "end $id exit=$code"
	[[ -n $once ]] && break
done
log "stopped"
