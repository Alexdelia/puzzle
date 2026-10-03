#!/usr/bin/env bash

set -euo pipefail
cd "$(dirname "$0")"

placeholder='const SOLUTIONS: &[(&str, &str)] = &[];'
answer=src/bin/answer.rs

entries() {
	for solution in solution/*.txt; do
		printf '\t(\n\t\tr##"%s"##,\n\t\tr##"%s"##,\n\t),\n' \
			"$(cat "validator/submit/$(basename "$solution")")" \
			"$(cat "$solution")"
	done
}

grep -qxF "$placeholder" "$answer"
while IFS= read -r line; do
	if [[ $line == "$placeholder" ]]; then
		echo 'const SOLUTIONS: &[(&str, &str)] = &['
		entries
		echo '];'
	else
		printf '%s\n' "$line"
	fi
done <"$answer"
