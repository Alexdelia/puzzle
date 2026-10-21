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

cd "$root"
worker_pid=$BASHPID
waiting=
while [[ ! -e $state/STOP ]]; do
	running=$("$tool_dir/next.sh" "$worker" "$worker_pid" $task)
	case $? in
	0) ;;
	1)
		[[ -n $once ]] && break
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
	read -r id task_model task_effort < <(jq -r '[.id, .model, .effort] | @tsv' "$running")
	log "start $id model=$task_model effort=$task_effort"
	output=$state/worker/$id
	"$tool_dir/prompt.sh" "$running" >"$output.prompt"
	CG_TASK_ID=$id command time -f '%e %U %S' -o "$output.time" \
		timeout --kill-after=60 "$task_wall_seconds" \
		systemd-run --user --scope --quiet --slice=codegolf.slice -p MemoryMax="$task_memory_max" \
		direnv exec "$root" claude -p "$(<"$output.prompt")" --model "$task_model" --effort "$task_effort" \
		--output-format stream-json --verbose --permission-mode auto --max-budget-usd "$task_budget_usd" \
		>"$output.jsonl" 2>"$output.err" </dev/null
	code=$?
	jq -c 'select(.type == "result")' "$output.jsonl" | tail -n 1 >"$output.json"
	record_usage "$output.jsonl"
	"$tool_dir/finish.sh" "$running" "$output.json" "$output.time" "$code"
	log "end $id exit=$code"
	[[ -n $once ]] && break
done
log "stopped"
