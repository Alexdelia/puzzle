#!/usr/bin/env bash
set -uo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: oracle.sh <puzzle/<p>/probe/<lang>/<name>.<ext>> <question> | oracle.sh --log <puzzle>"

if [[ ${1:-} == --log ]]; then
	log=$root/puzzle/${2:?$usage}/probe/probes.tsv
	[[ -s $log ]] && cat "$log" || echo "oracle.sh: no probe yet for $2"
	exit 0
fi

solution=${1:?$usage}
question=${2:?$usage}
question=${question//[$'\t\n']/ }
relative=$(realpath --relative-to="$root" "$solution")
[[ $relative =~ ^puzzle/([^/]+)/probe/([^/]+)/[^/]+$ ]] || { echo "$usage" >&2; exit 2; }
puzzle=${BASH_REMATCH[1]}
lang=${BASH_REMATCH[2]}
log=$root/puzzle/$puzzle/probe/probes.tsv
if [[ -s $log ]] && awk -F'\t' -v question="$question" -v lang="$lang" '$3 == lang && $5 == question { found = 1 } END { exit !found }' "$log"; then
	echo "oracle.sh: already asked in $lang, see oracle.sh --log $puzzle:" >&2
	awk -F'\t' -v question="$question" -v lang="$lang" '$3 == lang && $5 == question' "$log" >&2
	exit 2
fi

length=$("$tool_dir/count.sh" "$solution")
output=$("$tool_dir/submit.sh" --probe "$solution" 2>&1)
code=$?
printf '%s\n' "$output"
case $code in
0) answer="yes: 100% at length $length, committed" ;;
3) answer=$(grep -o 'progress .*' <<<"$output" | head -n 1) ;;
*) echo "oracle.sh: probe not sent (submit.sh exit $code), nothing logged" >&2 && exit "$code" ;;
esac
with_lock oracle bash -c 'printf "%s\t%s\t%s\t%s\t%s\t%s\n" "$@" >>"$0"' "$log" \
	"$(now)" "${CG_TASK_ID:--}" "$lang" "$length" "$question" "$answer"
echo "oracle.sh: logged in puzzle/$puzzle/probe/probes.tsv"
