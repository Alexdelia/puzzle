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

cd "$root"
while [[ ! -e $state/STOP ]]; do
	best= best_gain=0 estimated=0 estimate_started=$EPOCHSECONDS
	while IFS=$'\t' read -r job queued_at gain; do
		estimated=$((estimated + 1))
		if ((gain == 0)); then
			remove_unless_requeued "$job" "$queued_at"
			event "wrap nothing $(jq -r .puzzle "$job")"
			continue
		fi
		((gain > best_gain)) && best=$job best_gain=$gain
	done < <(ready_jobs | xargs -r -P "$wrap_estimate_jobs" -I{} bash -c 'estimate_job "$1"' _ {})
	((estimated == 0)) || log "estimated $estimated jobs in $((EPOCHSECONDS - estimate_started))s"
	if [[ -z $best ]]; then
		sleep 60
		continue
	fi
	puzzle=$(jq -r .puzzle "$best")
	queued_at=$(jq -r .queued_at "$best")
	log "run $puzzle estimated gain $best_gain"
	run_started=$EPOCHSECONDS
	"$tool_dir/wrap.sh" "$puzzle" --submit | while IFS= read -r line; do log "$puzzle $line"; done
	code=${PIPESTATUS[0]}
	log "$puzzle finished exit $code in $((EPOCHSECONDS - run_started))s"
	if ((code == 0)); then
		remove_unless_requeued "$best" "$queued_at"
	else
		defer_unless_requeued "$best" "$queued_at"
		log "$puzzle deferred (exit $code), retry in ${wrap_retry_seconds}s"
	fi
	event "wrap run $puzzle estimated_gain=$best_gain exit=$code"
done
log "stopped"
