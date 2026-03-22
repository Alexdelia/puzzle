#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

VALIDATOR_DIR="$SCRIPT_DIR/validator"
OUTPUT_DIR="$SCRIPT_DIR/output"

SOLUTION_FILE_NAME="solution.txt"
VALIDATOR_FILE_EXTENSION=".txt"

S=""

for validator in "$OUTPUT_DIR"/*; do
	validator_name=$(basename "$validator")

	solution_file="$validator/$SOLUTION_FILE_NAME"

	if [[ -f "$solution_file" ]]; then
		flag="$(head -n 1 "$VALIDATOR_DIR/$validator_name$VALIDATOR_FILE_EXTENSION")"
		solution="$(cat "$solution_file")"
		solution="${solution//$'\n'/';'}"
		S+="(
\"$flag\",
r#\"$solution\"#
),
"
	fi
done

printf "%s" "$S"
