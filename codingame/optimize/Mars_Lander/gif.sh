#!/usr/bin/env bash

set -xeuo pipefail

VALIDATOR="${1:-}"

if [[ -z "$VALIDATOR" ]]; then
	printf "usage:\t$0 <validator-name>\n"
	exit 64
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
cd "$SCRIPT_DIR/output/$VALIDATOR/visualize"

OUTPUT_FILE="./animation.gif"

magick -delay 10 -loop 0 *.svg "$OUTPUT_FILE"

xdg-open "$OUTPUT_FILE" &>/dev/null
