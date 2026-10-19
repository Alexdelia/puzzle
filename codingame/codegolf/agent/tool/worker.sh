#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"

worker=${1:?usage: worker.sh <worker-id> [--once]}
once=${2:-}
log() {
	echo "$(now) worker $worker: $*"
}

cd "$root"
while [[ ! -e $state/STOP ]]; do
	running=$("$tool_dir/next.sh" "$worker" "$BASHPID")
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
	*)
		log "next.sh failed"
		sleep 60
		continue
		;;
	esac
	id=$(jq -r .id "$running")
	log "start $id"
	output=$state/worker/$id
	"$tool_dir/prompt.sh" "$running" >"$output.prompt"
	CG_TASK_ID=$id command time -f '%e %U %S' -o "$output.time" \
		timeout --kill-after=60 "$task_wall_seconds" \
		direnv exec "$root" claude -p "$(<"$output.prompt")" --model "$model" --effort "$effort" \
		--output-format stream-json --verbose --permission-mode dontAsk --max-budget-usd "$task_budget_usd" \
		>"$output.jsonl" 2>"$output.err" </dev/null
	code=$?
	jq -c 'select(.type == "result")' "$output.jsonl" | tail -n 1 >"$output.json"
	record_usage "$output.jsonl"
	"$tool_dir/finish.sh" "$running" "$output.json" "$output.time" "$code"
	log "end $id exit=$code"
	[[ -n $once ]] && break
done
log "stopped"
