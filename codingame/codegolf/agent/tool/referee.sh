#!/usr/bin/env bash
set -euo pipefail

usage="usage: run.sh <solution-file> [--lang <lang>] [--bench] [referee options] [test...]"
referee_dir=$(cd "${1:?$usage}" && pwd)
shift
solution=${1:?$usage}
shift
tool_dir=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$tool_dir/../.." && pwd)
slug=$(basename "$(dirname "$referee_dir")")

lang=$(basename "$(dirname "$(realpath "$solution")")")
bench=false
referee_args=()
while (($#)); do
	case $1 in
	--lang)
		lang=$2
		shift 2
		;;
	--bench)
		bench=true
		shift
		;;
	*)
		referee_args+=("$1")
		shift
		;;
	esac
done
[[ -x $tool_dir/lang/$lang/build.sh ]] || { echo "run.sh: unknown lang $lang, pass --lang" >&2; exit 2; }

export CARGO_TARGET_DIR=$root/agent/state/build/cargo/$slug
cargo build --release --quiet --manifest-path "$referee_dir/Cargo.toml"
referee=$CARGO_TARGET_DIR/release/$(basename "$(dirname "$referee_dir")")-referee
launcher=$("$tool_dir/lang/$lang/build.sh" "$solution")

if $bench; then
	"$referee" "$launcher" --root "$referee_dir" --validators --bench 5 "${referee_args[@]}" | tee "$referee_dir/bench.txt"
	exit "${PIPESTATUS[0]}"
fi
status=0
"$referee" "$launcher" --root "$referee_dir" "${referee_args[@]}" || status=$?
echo "length $("$tool_dir/count.sh" "$solution")"
exit "$status"
