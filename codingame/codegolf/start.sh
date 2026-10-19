#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh
shopt -s nullglob

usage="usage: start.sh [--workers n] [--once]"
count=$workers
once=
while (($#)); do
	case $1 in
	--workers) count=$2 && shift 2 ;;
	--once) once=--once && shift ;;
	*) echo "$usage" >&2 && exit 2 ;;
	esac
done

if [[ -z ${CG_PYTHON:-} ]]; then
	exec direnv exec "$root" "$0" "$@"
fi

rm -f "$state/STOP"
for running in "$state"/running/*.json; do
	pid=$(jq -r '.worker_pid // 0' "$running")
	if ! kill -0 "$pid" 2>/dev/null; then
		read -r id kind puzzle lang < <(jq -r '[.id, .kind, .puzzle, .lang] | @tsv' "$running")
		jq 'del(.started_at, .worker, .worker_pid, .budget_usd, .outcome, .note)' "$running" >"$state/queue/$id.json"
		rm -f "$running"
		json_update "$(ensure_status "$kind" "$puzzle" "$lang")" '.status = "todo"'
		event "requeue orphan $id"
	fi
done
agent/tool/sync.sh --if-stale

event "start workers=$count ${once:+once}"
pids=()
for ((worker = 1; worker <= count; worker++)); do
	setsid agent/tool/worker.sh "$worker" $once >>"$state/worker/worker-$worker.log" 2>&1 &
	pids+=($!)
	sleep 2
done
echo "start.sh: $count workers, logs in agent/state/worker/worker-<n>.log, stop with ./stop.sh"
wait "${pids[@]}"
event "all workers stopped"
