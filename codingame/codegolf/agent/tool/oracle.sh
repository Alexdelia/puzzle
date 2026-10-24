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
CG_PROBE_QUESTION=$question "$tool_dir/submit.sh" --probe "$solution"
code=$?
case $code in
0 | 3) echo "oracle.sh: logged in puzzle/$puzzle/probe/probes.tsv" ;;
*) echo "oracle.sh: submit.sh exit $code: if it got as far as the POST, its row in puzzle/$puzzle/probe/probes.tsv says pending with the submission id" >&2 && exit "$code" ;;
esac
