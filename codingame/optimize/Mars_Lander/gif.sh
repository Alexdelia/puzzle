#!/usr/bin/env bash

set -euo pipefail

VALIDATOR="${1:-}"

if [[ -z "$VALIDATOR" ]]; then
	printf "usage:\t$0 <validator-name>\n"
	exit 64
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
cd "$SCRIPT_DIR/output/$VALIDATOR/visualize"

OUTPUT_FILE="./animation.gif"

MAX_FRAME=128

files=($(ls -1 *.svg | sort -n))
total="${#files[@]}"

declare -a selected_files
if [ "$total" -lt "$MAX_FRAME" ]; then
	selected_files=("${files[@]}")
else
	step=$(echo "scale=10; ($total - 1) / ($MAX_FRAME - 1)" | bc)

	selected_indices=()

	for i in $(seq 0 $((MAX_FRAME - 1))); do
		index=$(echo "$i * $step" | bc | awk '{printf "%.0f", $0}')
		selected_indices+=($index)
	done

	selected_indices+=($((total - 1)))

	selected_files=()
	for index in "${selected_indices[@]}"; do
		selected_files+=("${files[$index]}")
	done
fi

magick -delay 10 -loop 0 ${selected_files[@]} "$OUTPUT_FILE"

xdg-open "$OUTPUT_FILE" &>/dev/null
