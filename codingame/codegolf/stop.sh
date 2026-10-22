#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh
shopt -s nullglob

kill_session() {
	[[ $1 =~ ^[0-9]+$ && $(ps -o sid= -p "$1" | tr -d ' ') == "$1" ]] && pkill -TERM -s "$1"
	true
}

touch "$state/STOP"
event "stop ${1:-}"
if [[ ${1:-} != --now ]]; then
	echo "stop.sh: no new task will start; running tasks finish (./stop.sh --now kills them)"
	exit 0
fi
for running in "$state"/running/*.json; do
	read -r id kind puzzle lang pid < <(jq -r '[.id, .kind, .puzzle, .lang, .worker_pid] | @tsv' "$running")
	kill_session "$pid"
	jq 'del(.started_at, .worker, .worker_pid, .outcome, .note)' "$running" >"$state/queue/$id.json"
	rm -f "$running"
	json_update "$(ensure_status "$kind" "$puzzle" "$lang")" '.status = "todo"'
	event "requeue killed $id"
done
for session in $(pgrep -f 'agent/tool/(worker|wrapd).sh'); do
	kill_session "$session"
done
echo "stop.sh: workers killed, their tasks back in queue, journals kept"
