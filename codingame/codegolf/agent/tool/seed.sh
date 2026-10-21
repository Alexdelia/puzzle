#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: seed.sh env|validators|puzzles|cells|reviews [puzzle...|lang...]"
queued() {
	grep -q -F "\"key\": \"$1\"" "$state"/queue/*.json "$state"/running/*.json 2>/dev/null
}
seed() {
	local kind=$1 puzzle=$2 lang=$3 finished=$4 status
	status=$(jq -r .status "$(ensure_status "$kind" "$puzzle" "$lang")")
	if [[ $status != "$finished" && $status != blocked ]] && ! queued "$kind/$puzzle/$lang"; then
		"$tool_dir/enqueue.sh" "$kind" "$puzzle" "$lang" --reason seed
	fi
}
case ${1:?$usage} in
env)
	for lang in $(langs); do seed lang - "$lang" ready; done
	for puzzle in $(puzzles); do seed referee "$puzzle" - done; done
	;;
validators)
	shift
	for puzzle in ${*:-$(puzzles)}; do seed validator "$puzzle" - done; done
	;;
puzzles)
	for puzzle in $(puzzles); do seed puzzle "$puzzle" - done; done
	;;
cells)
	for puzzle in $(puzzles); do
		for lang in $(langs); do seed cell "$puzzle" "$lang" first; done
	done
	;;
reviews)
	shift
	for lang in ${*:-$(langs)}; do seed review - "$lang" done; done
	;;
*) echo "$usage" >&2 && exit 2 ;;
esac
