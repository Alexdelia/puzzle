#!/usr/bin/env bash
set -euo pipefail

file=${1:?usage: count.sh <file>}
bytes=$(wc -c <"$file")
while ((bytes > 0)) && [[ $(tail -c "+$bytes" "$file" | head -c 1 | od -An -tx1 | tr -d ' ') == 0a ]]; do
	((bytes--))
done
echo "$bytes"
