#!/usr/bin/env bash
set -euo pipefail

readonly CODINGAME_CHARACTER_LIMIT=100000
here=$(cd "$(dirname "$0")" && pwd)
bot=${1:?usage: bundle.sh <bot name, e.g. apex> [output file]}
source="$here/$bot.rs"
output=${2:-$here/../target/bundle/$bot.rs}
[[ -f $source ]] || { echo "no bot at $source" >&2; exit 1; }

mkdir -p "$(dirname "$output")"
{
	echo "#[allow(dead_code)]"
	echo "mod engine {"
	cat "$here/engine.rs"
	echo "}"
	echo
	cat "$source"
} >"$output"

scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
rustc --edition 2021 -O --crate-type bin -o "$scratch/$bot" "$output"

size=$(wc -m <"$output")
if ((size > CODINGAME_CHARACTER_LIMIT)); then
	echo "$output: $size characters, over the CodinGame limit of $CODINGAME_CHARACTER_LIMIT" >&2
	exit 1
fi
echo "$output: $size characters"
