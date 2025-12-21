#!/usr/bin/env bash

set -xeuo pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

OUTPUT_DIR="$SCRIPT_DIR/output"
mkdir -p "$OUTPUT_DIR"

MERGED_OUTPUT="$OUTPUT_DIR/merged.txt"
> "$MERGED_OUTPUT"

cargo build --release --bin solver
SOLVER_PATH="$SCRIPT_DIR/target/release/solver"

set +x

for input_file in "$SCRIPT_DIR"/validator/*.txt; do
	start="$(date +%s%N)"
	output_file="$OUTPUT_DIR/$(basename "$input_file")"

	"$SOLVER_PATH" < "$input_file" > "$output_file"

	end="$(date +%s%N)"

	sed '2q;d' "$input_file" | tr -d '\n' >> "$MERGED_OUTPUT"
	printf ':' >> "$MERGED_OUTPUT"
	cat "$output_file" >> "$MERGED_OUTPUT"

	elapsed="$((end - start))"
	input_name="$(basename "$input_file")"
	input_name="${input_name%.txt}"
	printf "${input_name}:\t%.3fs\n" "${elapsed}e-9"
done

cat "$MERGED_OUTPUT"
