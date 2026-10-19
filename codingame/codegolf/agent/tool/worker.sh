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
		log "budget cap reached, waiting"
		sleep 300
		continue
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
		claude -p "$(<"$output.prompt")" --model "$model" --effort "$effort" --output-format json \
		--permission-mode dontAsk --max-budget-usd "$task_budget_usd" >"$output.json" 2>"$output.err" </dev/null
	code=$?
	"$tool_dir/finish.sh" "$running" "$output.json" "$output.time" "$code"
	log "end $id exit=$code"
	[[ -n $once ]] && break
done
log "stopped"
