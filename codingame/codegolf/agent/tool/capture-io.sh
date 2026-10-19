#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C.UTF-8

slug=${1:?usage: capture-io.sh <puzzle-slug>}
tool_dir=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$tool_dir/../.." && pwd)
pretty_id=$(awk -F'\t' -v slug="$slug" '$1 == slug { print $2 }' "$root/data/puzzle.tsv")
[[ -n $pretty_id ]] || { echo "capture-io.sh: unknown puzzle $slug" >&2; exit 2; }
tests_dir=$root/puzzle/$slug/referee/tests
mkdir -p "$tests_dir"

echo_program='import sys;d=sys.stdin.read();sys.stderr.write(f"{len(d)}\n{d}")'
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

count=$("$tool_dir/cg.sh" GET "/puzzles/$pretty_id" | jq '.testcases | length')
for ((index = 1; index <= count; index++)); do
	jq -n --arg code "$echo_program" --argjson testcase "$index" \
		'{language: "Python3", code: $code, testcase: $testcase}' >"$work/body.json"
	"$tool_dir/cg.sh" POST "/puzzles/$pretty_id/test-games" "$work/body.json" | jq -j '.output' >"$work/output"
	declared=$(head -n 1 "$work/output")
	tail -n +2 "$work/output" >"$work/input"
	captured=$(wc -m <"$work/input")
	if ((captured + 1 == declared)); then
		echo >>"$work/input"
		captured=$((captured + 1))
	fi
	name=$(printf '%02d' "$index")
	if ((captured != declared)); then
		echo "capture-io.sh: test $name truncated: $captured of $declared chars" >&2
		exit 1
	fi
	mv "$work/input" "$tests_dir/$name.in"
	echo "test $name captured ($declared chars)"
done
