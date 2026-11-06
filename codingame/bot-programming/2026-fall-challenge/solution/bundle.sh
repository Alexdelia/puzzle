#!/usr/bin/env bash
set -euo pipefail

readonly CODINGAME_CHARACTER_LIMIT=100000
here=$(cd "$(dirname "$0")" && pwd)
usage="usage: bundle.sh <bot>[/<variant>, default best] [output file] [NAME=VALUE overriding DEFAULT_NAME ...]"
spec=${1:?$usage}
bot=${spec%%/*}
variant=${spec#"$bot"}
variant=${variant#/}
source="$here/$bot.rs"
output=${2:-$here/../target/bundle/${spec//\//_}.rs}
shift $(($# < 2 ? $# : 2))
if [[ -n ${WEIGHT_DIR:-} ]]; then
	weights=$WEIGHT_DIR
elif [[ -n $variant ]]; then
	weights="$here/../weight/$bot/$variant"
	[[ -d $weights ]] || { echo "no variant at $weights" >&2; exit 1; }
else
	weights="$here/../weight/$bot/best"
fi
settings="$weights/settings"
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

overrides=()
if [[ -f $settings ]]; then
	mapfile -t overrides < <(grep -v '^[[:space:]]*$' "$settings")
fi
overrides+=("$@")

python3 - "$output" "$weights" "${overrides[@]}" <<'PYTHON'
import pathlib
import re
import sys

PACK_BASE = 0x4E00
PACK_BITS = 14


def pack(payload):
	out = []
	bits = 0
	have = 0
	for byte in payload:
		bits = (bits << 8) | byte
		have += 8
		while have >= PACK_BITS:
			have -= PACK_BITS
			out.append(chr(PACK_BASE + ((bits >> have) & 0x3FFF)))
	if have:
		out.append(chr(PACK_BASE + ((bits << (PACK_BITS - have)) & 0x3FFF)))
	return "".join(out)


def replace_once(body, pattern, replacement, missing):
	body, replaced = re.subn(pattern, replacement, body, count=1)
	if replaced != 1:
		sys.exit(missing)
	return body


output = pathlib.Path(sys.argv[1])
weight_dir = pathlib.Path(sys.argv[2])
body = output.read_text(encoding="utf-8")
for weight in sorted(weight_dir.glob("*.bin")) if weight_dir.is_dir() else []:
	name = f"{weight.stem.upper()}_WEIGHTS"
	body = replace_once(
		body,
		rf'pub const {name}: &str = "[^"]*";',
		lambda _: f'pub const {name}: &str = "{pack(weight.read_bytes())}";',
		f"{weight}: the bot has no constant {name}",
	)
	print(f"{weight.name}: packed into {name}")
for setting in sys.argv[3:]:
	name, value = setting.split("=", 1)
	body = replace_once(
		body,
		rf"pub const DEFAULT_{name}: ([^=]*) = [^;]*;",
		lambda match: f"pub const DEFAULT_{name}: {match[1]} = {value};",
		f"{setting}: the bot has no constant DEFAULT_{name}",
	)
body = "\n".join(line.lstrip() for line in body.splitlines()) + "\n"
output.write_text(body, encoding="utf-8")
PYTHON

scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
rustc --edition 2021 -O --crate-type bin --crate-name bot -o "$scratch/$bot" "$output"

size=$(python3 -c 'import sys; print(len(open(sys.argv[1], encoding="utf-8").read().encode("utf-16-le")) // 2)' "$output")
if ((size > CODINGAME_CHARACTER_LIMIT)); then
	echo "$output: $size UTF-16 units, over the CodinGame limit of $CODINGAME_CHARACTER_LIMIT" >&2
	exit 1
fi
echo "$output: $size UTF-16 units"
