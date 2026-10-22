#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh
shopt -s nullglob

usage="usage: start.sh [--workers n] [--once] [--best] [--full-window]"
arguments=("$@")
count=$workers
once=
best=
full_window=
while (($#)); do
	case $1 in
	--workers) count=$2 && shift 2 ;;
	--once) once=--once && shift ;;
	--best) best=1 && shift ;;
	--full-window) full_window=1 && shift ;;
	*) echo "$usage" >&2 && exit 2 ;;
	esac
done

if [[ -z ${CG_PYTHON:-} ]]; then
	exec direnv exec "$root" "$0" "${arguments[@]}"
fi

refresh_usage
if over_weekly_pace; then
	read -r used elapsed < <(weekly_pace)
	echo "start.sh: weekly usage ${used}% >= ${elapsed}% of the week elapsed (margin ${weekly_pace_margin_percent}%): not starting" >&2
	exit 1
fi
alive=$(loop_pids 'worker|wrapd|wrap' | wc -l)
if ((alive)); then
	echo "start.sh: $alive worker loops still alive (a soft stop lets them finish their task first): wait for them to exit, or ./stop.sh --now" >&2
	exit 1
fi
rm -f "$state/STOP"
commit_time_log
[[ -n $(ls "$state"/lang/*.json 2>/dev/null) ]] || agent/tool/rebuild.sh
for running in "$state"/running/*.json; do
	pid=$(jq -r '.worker_pid // 0' "$running")
	if ! kill -0 "$pid" 2>/dev/null; then
		read -r id kind puzzle lang < <(jq -r '[.id, .kind, .puzzle, .lang] | @tsv' "$running")
		jq 'del(.started_at, .worker, .worker_pid, .outcome, .note)' "$running" >"$state/queue/$id.json"
		rm -f "$running"
		json_update "$(ensure_status "$kind" "$puzzle" "$lang")" '.status = "todo"'
		event "requeue orphan $id"
	fi
done
agent/tool/sync.sh --if-stale
agent/tool/seed.sh wrappers >/dev/null
agent/tool/seed.sh wraps >/dev/null

systemctl --user set-property --runtime codegolf.slice MemoryMax="$workers_memory_max" MemorySwapMax=infinity
export CG_BEST_MODEL=$best CG_FULL_WINDOW=$full_window
event "start workers=$count ${once:+once} ${best:+best} ${full_window:+full-window}"
pids=()
[[ -n $once ]] || setsid agent/tool/wrapd.sh >>"$state/worker/wrapd.log" 2>&1 &
for ((worker = 1; worker <= count; worker++)); do
	setsid agent/tool/worker.sh "$worker" $once >>"$state/worker/worker-$worker.log" 2>&1 &
	pids+=($!)
	sleep 2
done
echo "start.sh: $count workers${best:+, best model for every task}${full_window:+, five-hour window used up to 100%}, logs in agent/state/worker/worker-<n>.log, stop with ./stop.sh"
wait "${pids[@]}" || true
event "all workers stopped"
commit_time_log
