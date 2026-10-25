#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

log() {
	echo "$(now) wrapd: $*"
}

estimated_gain() {
	"$tool_dir/wrap.sh" "$1" --estimate 2>/dev/null |
		awk -F'\t' 'NF == 3 && $3 != "-" { gain += ($2 == "-") ? no_row_gain : ($2 > $3 ? $2 - $3 : 0) } END { print gain + 0 }' no_row_gain="$wrap_no_row_gain"
}

estimate_job() {
	local job=$1 queued_at puzzle
	queued_at=$(jq -r .queued_at "$job")
	puzzle=$(jq -r .puzzle "$job")
	printf '%s\t%s\t%s\n' "$job" "$queued_at" "$(estimated_gain "$puzzle")"
}
export -f estimate_job estimated_gain
export tool_dir wrap_no_row_gain

ready_jobs() {
	local job
	for job in "$state"/wrapq/*.json; do
		(($(jq '.retry_after // 0' "$job") <= $(date +%s))) && printf '%s\n' "$job"
	done
}

remove_unless_requeued() {
	with_lock wrapq bash -c '[[ $(jq -r .queued_at "$1" 2>/dev/null) == "$2" ]] && rm -f "$1"; true' _ "$1" "$2"
}

defer_unless_requeued() {
	with_lock wrapq bash -c '
		[[ $(jq -r .queued_at "$1" 2>/dev/null) == "$2" ]] || exit 0
		jq --argjson at "$3" ".retry_after = \$at" "$1" >"$1.tmp" && mv "$1.tmp" "$1"' _ "$1" "$2" "$(($(date +%s) + wrap_retry_seconds))"
}

run_job() {
	local puzzle=$1
	"$tool_dir/wrap.sh" "$puzzle" --submit | while IFS= read -r line; do log "$puzzle $line"; done
	return "${PIPESTATUS[0]}"
}

declare -A running_pid=() running_job=() running_queued=() running_gain=() running_started=()

start_jobs() {
	local free=$1 job queued_at gain puzzle started=0 estimated=0 estimate_started=$EPOCHSECONDS
	local -a ranked=()
	while IFS=$'\t' read -r job queued_at gain; do
		estimated=$((estimated + 1))
		if ((gain == 0)); then
			puzzle=$(jq -r .puzzle "$job")
			remove_unless_requeued "$job" "$queued_at"
			event "wrap nothing $puzzle"
			continue
		fi
		ranked+=("$gain"$'\t'"$job"$'\t'"$queued_at")
	done < <(idle_jobs | xargs -r -P "$wrap_estimate_jobs" -I{} bash -c 'estimate_job "$1"' _ {})
	((estimated == 0)) || log "estimated $estimated jobs in $((EPOCHSECONDS - estimate_started))s"
	while IFS=$'\t' read -r gain job queued_at; do
		((started < free)) || break
		puzzle=$(jq -r .puzzle "$job")
		if ! with_lock queue claim_puzzle "$puzzle"; then
			log "skip $puzzle: task of it claimed meanwhile"
			continue
		fi
		log "run $puzzle estimated gain $gain"
		run_job "$puzzle" </dev/null &
		running_pid[$puzzle]=$! running_job[$puzzle]=$job running_queued[$puzzle]=$queued_at
		running_gain[$puzzle]=$gain running_started[$puzzle]=$EPOCHSECONDS
		started=$((started + 1))
	done < <(((${#ranked[@]})) && printf '%s\n' "${ranked[@]}" | sort -t$'\t' -k1,1nr)
}

idle_jobs() {
	local job puzzle
	local -A task_running=()
	while IFS= read -r puzzle; do task_running[$puzzle]=1; done < <(puzzles_with_running_tasks)
	while IFS= read -r job; do
		puzzle=$(jq -r .puzzle "$job")
		[[ -z ${running_pid[$puzzle]:-} && -z ${task_running[$puzzle]:-} ]] && printf '%s\n' "$job"
	done < <(ready_jobs)
}

claim_puzzle() {
	local puzzles
	puzzles=$(puzzles_with_running_tasks)
	grep -qxF "$1" <<<"$puzzles" && return 1
	touch "$wrap_running_dir/$1"
}

queue_signature() {
	{ idle_jobs | xargs -r jq -r .queued_at; } | sort | md5sum
}

finish_job() {
	local puzzle=$1 code=$2
	log "$puzzle finished exit $code in $((EPOCHSECONDS - running_started[$puzzle]))s"
	if ((code == 0)); then
		remove_unless_requeued "${running_job[$puzzle]}" "${running_queued[$puzzle]}"
	else
		defer_unless_requeued "${running_job[$puzzle]}" "${running_queued[$puzzle]}"
		log "$puzzle deferred (exit $code), retry in ${wrap_retry_seconds}s"
	fi
	event "wrap run $puzzle estimated_gain=${running_gain[$puzzle]} exit=$code"
	rm -f "$wrap_running_dir/$puzzle"
	unset "running_pid[$puzzle]" "running_job[$puzzle]" "running_queued[$puzzle]" "running_gain[$puzzle]" "running_started[$puzzle]"
}

cd "$root"
rm -rf "$wrap_running_dir"
mkdir -p "$wrap_running_dir"
estimated_signature= estimated_at=0
while [[ ! -e $state/STOP ]]; do
	free=$((wrap_puzzles - ${#running_pid[@]}))
	signature=$(queue_signature)
	if ((free > 0)) && [[ $signature != "$estimated_signature" ]] || ((free > 0 && EPOCHSECONDS - estimated_at >= wrap_retry_seconds)); then
		start_jobs "$free"
		estimated_signature=$(queue_signature) estimated_at=$EPOCHSECONDS
	fi
	sleep 60 &
	timer=$!
	finished=
	wait -n -p finished "$timer" "${running_pid[@]}"
	code=$?
	if [[ $finished == "$timer" ]]; then
		continue
	fi
	kill "$timer" 2>/dev/null
	for puzzle in "${!running_pid[@]}"; do
		[[ ${running_pid[$puzzle]} == "$finished" ]] && finish_job "$puzzle" "$code"
	done
	estimated_signature=
done
for puzzle in "${!running_pid[@]}"; do
	wait "${running_pid[$puzzle]}"
	finish_job "$puzzle" "$?"
done
log "stopped"
