#!/usr/bin/env bash
set -euo pipefail

usage="usage: expect-io.sh <puzzle-slug> <reference-solution>"
slug=${1:?$usage}
solution=${2:?$usage}
tool_dir=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$tool_dir/../.." && pwd)
pretty_id=$(awk -F'\t' -v slug="$slug" '$1 == slug { print $2 }' "$root/data/puzzle.tsv")
lang=$(basename "$(dirname "$solution")")
cg_name=$(awk -F'\t' -v slug="$lang" '$1 == slug { print $2 }' "$root/data/lang.tsv")
[[ -n $pretty_id && -n $cg_name ]] || { echo "expect-io.sh: unknown puzzle $slug or lang $lang" >&2; exit 2; }
tests_dir=$root/puzzle/$slug/referee/tests
launcher=$("$tool_dir/lang/$lang/build.sh" "$solution")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

for input in "$tests_dir"/*.in; do
	name=$(basename "$input" .in)
	jq -n --rawfile code "$solution" --arg language "$cg_name" --argjson testcase "$((10#$name))" \
		'{language: $language, code: $code, testcase: $testcase}' >"$work/body.json"
	"$tool_dir/cg.sh" POST "/puzzles/$pretty_id/test-games" "$work/body.json" >"$work/result.json"
	if [[ $(jq .success "$work/result.json") != true ]]; then
		echo "expect-io.sh: reference fails CG test $name: $(jq -c '{found, expected, error, timeout}' "$work/result.json")" >&2
		exit 1
	fi
	timeout 10 "$launcher" <"$input" >"$tests_dir/$name.out"
	echo "test $name expected written"
done
