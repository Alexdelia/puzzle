#!/usr/bin/env bash
set -euo pipefail

usage="usage: run.sh <solution-file> [--lang <lang>] [--bench] [--cg] [referee options] [test...]; --cg: the cases CG grades, hidden/ only when hidden/complete exists, else tests + synthetic validators"
referee_dir=$(cd "${1:?$usage}" && pwd)
shift
solution=${1:?$usage}
shift
tool_dir=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$tool_dir/../.." && pwd)
slug=$(basename "$(dirname "$referee_dir")")

cg_graded_args() {
	local case found=
	if [[ -e $referee_dir/hidden/complete ]]; then
		for case in "$referee_dir"/hidden/*.in "$referee_dir"/hidden/*.json; do
			[[ -e $case ]] || continue
			printf 'hidden/%s\n' "$(basename "${case%.*}")"
			found=1
		done
	fi
	[[ -n $found ]] && echo --hidden || echo --validators
}

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
	--cg)
		mapfile -t -O "${#referee_args[@]}" referee_args < <(cg_graded_args)
		shift
		;;
	*)
		referee_args+=("$1")
		shift
		;;
	esac
done
[[ -x $tool_dir/lang/$lang/build.sh ]] || { echo "run.sh: unknown lang $lang, pass --lang" >&2; exit 2; }

export CG_PUZZLE_KIND
CG_PUZZLE_KIND=$(awk -F'\t' -v slug="$slug" '$1 == slug { print $4 }' "$root/data/puzzle.tsv")
export CARGO_TARGET_DIR=$root/agent/state/build/cargo/$slug
cargo build --release --quiet --manifest-path "$referee_dir/Cargo.toml"
referee=$CARGO_TARGET_DIR/release/$(basename "$(dirname "$referee_dir")")-referee
build_status=0
launcher=$("$tool_dir/lang/$lang/build.sh" "$solution") || build_status=$?
if ((build_status)); then
	smoke=$(compgen -G "$tool_dir/lang/$lang/smoke/main.*" | head -n 1)
	if ((build_status < 128)) && "$tool_dir/lang/$lang/build.sh" "$smoke" >/dev/null 2>&1; then
		echo "build fail exit $build_status"
	else
		echo "build broken exit $build_status: toolchain fails on smoke/${smoke##*/} too"
	fi
	exit 1
fi
baseline_args=()
for baseline_source in "$tool_dir/lang/$lang"/baseline.*; do
	[[ -f $baseline_source ]] && baseline_args=(--baseline "$("$tool_dir/lang/$lang/build.sh" "$baseline_source")")
done

if $bench; then
	report=$(mktemp)
	status=0
	"$referee" "$launcher" --root "$referee_dir" --validators --bench 5 "${baseline_args[@]}" "${referee_args[@]}" | tee "$report" || status=${PIPESTATUS[0]}
	if [[ ${CG_TASK_ID:-referee.} == referee.* ]] && grep -q '^suite ' "$report"; then
		mv "$report" "$referee_dir/bench.txt"
	else
		rm -f "$report"
	fi
	exit "$status"
fi
status=0
"$referee" "$launcher" --root "$referee_dir" "${baseline_args[@]}" "${referee_args[@]}" || status=$?
echo "length $("$tool_dir/count.sh" "$solution")"
exit "$status"
