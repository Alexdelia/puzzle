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
	best= best_gain=0
	for job in "$state"/wrapq/*.json; do
		(($(jq '.retry_after // 0' "$job") <= $(date +%s))) || continue
		puzzle=$(jq -r .puzzle "$job")
		queued_at=$(jq -r .queued_at "$job")
		gain=$(estimated_gain "$puzzle")
		if ((gain == 0)); then
			remove_unless_requeued "$job" "$queued_at"
			event "wrap nothing $puzzle"
			continue
		fi
		((gain > best_gain)) && best=$job best_gain=$gain
	done
	if [[ -z $best ]]; then
		sleep 60
		continue
	fi
	puzzle=$(jq -r .puzzle "$best")
	queued_at=$(jq -r .queued_at "$best")
	log "run $puzzle estimated gain $best_gain"
	"$tool_dir/wrap.sh" "$puzzle" --submit > >(while IFS= read -r line; do log "$puzzle $line"; done)
	code=$?
	if ((code == 0)); then
		remove_unless_requeued "$best" "$queued_at"
	else
		defer_unless_requeued "$best" "$queued_at"
		log "$puzzle deferred (exit $code), retry in ${wrap_retry_seconds}s"
	fi
	event "wrap run $puzzle estimated_gain=$best_gain exit=$code"
done
log "stopped"
