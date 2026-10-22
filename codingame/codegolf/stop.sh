#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh
shopt -s nullglob

touch "$state/STOP"
event "stop ${1:-}"
if [[ ${1:-} != --now ]]; then
	echo "stop.sh: no new task will start; running tasks finish (./stop.sh --now kills them)"
	exit 0
fi
for running in "$state"/running/*.json; do
	read -r id kind puzzle lang pid < <(jq -r '[.id, .kind, .puzzle, .lang, .worker_pid] | @tsv' "$running")
	kill -TERM -- "-$pid" 2>/dev/null || true
	jq 'del(.started_at, .worker, .worker_pid, .outcome, .note)' "$running" >"$state/queue/$id.json"
	rm -f "$running"
	json_update "$(ensure_status "$kind" "$puzzle" "$lang")" '.status = "todo"'
	event "requeue killed $id"
done
pkill -f 'agent/tool/worker.sh' 2>/dev/null || true
pkill -f 'agent/tool/wrapd.sh' 2>/dev/null || true
echo "stop.sh: workers killed, their tasks back in queue, journals kept"
