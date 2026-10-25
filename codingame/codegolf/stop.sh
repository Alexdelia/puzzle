#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh
shopt -s nullglob

usage="usage: stop.sh [--wrap] [--now]; default: no new task nor wrap run starts, running ones finish; --wrap: wrap generator only, workers untouched; --now: kill instead of letting them finish"
now=
wrap_only=
for argument in "$@"; do
	case $argument in
	--now) now=1 ;;
	--wrap) wrap_only=1 ;;
	*) echo "$usage" >&2 && exit 2 ;;
	esac
done

kill_session() {
	local session
	[[ $1 =~ ^[0-9]+$ ]] || return 0
	session=$(ps -o sid= -p "$1" | tr -d ' ')
	[[ -n $session ]] || return 0
	if [[ $session == "$1" || $(ps -o comm= -p "$session") == bwrap ]]; then
		pkill -TERM -s "$session"
	fi
	true
}

if [[ -n $wrap_only ]]; then
	touch "$state/STOP_WRAP"
	event "stop $*"
	if [[ -z $now ]]; then
		echo "stop.sh: wrap generator starts no new run; running ones finish (./stop.sh --wrap --now kills them), workers untouched"
		exit 0
	fi
	for session in $(loop_pids wrapd); do
		kill_session "$session"
	done
	echo "stop.sh: wrap generator killed, its jobs kept in queue, workers untouched"
	exit 0
fi

touch "$state/STOP"
event "stop $*"
if [[ -z $now ]]; then
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
for session in $(loop_pids 'worker|wrapd'); do
	kill_session "$session"
done
systemctl --user stop 'codegolf-task-*.scope'
echo "stop.sh: workers killed, their tasks back in queue, journals kept"
