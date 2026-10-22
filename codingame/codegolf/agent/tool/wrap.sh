#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

usage="usage: wrap.sh <puzzle> [--host <lang>] [--estimate | --submit]; wraps every core solution of the puzzle (perl, python, bash...) in every host lang of agent/wrap/, keeps per host the shortest candidate under our row that passes the local referee; --estimate: lengths only, no referee; --submit: submit.sh the winners; exit 3: some host deferred (busy cell, submit cap, generator error)"
puzzle=${1:?$usage}
shift
host_filter=
mode=check
while (($#)); do
	case $1 in
	--host) host_filter=${2:?$usage} && shift 2 ;;
	--estimate) mode=estimate && shift ;;
	--submit) mode=submit && shift ;;
	*) echo "$usage" >&2 && exit 2 ;;
	esac
done
[[ -n $(tsv_field puzzle "$puzzle" slug) ]] || { echo "wrap.sh: unknown puzzle $puzzle" >&2; exit 2; }
kind=$(tsv_field puzzle "$puzzle" kind)
wrap_root=$root/agent/wrap
mkdir -p "$state/build/wrap"
work=$(mktemp -d "$state/build/wrap/$puzzle.XXXXXX")
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/derived"
local_sets=(--validators)
[[ -e $root/puzzle/$puzzle/referee/hidden/complete ]] && local_sets=(--hidden)
deferred=

derived_perl_core() {
	local file=$1 text
	text=$(<"$file")
	[[ $text != *$'\n'* && $text =~ ^perl((\ -[A-Za-z0-9]+)+)\'([^\']*)\'$ ]] || return 0
	local derived=$work/derived/$(basename "$(dirname "$file")")-$(basename "$file").pl
	printf '#!perl%s\n%s' "${BASH_REMATCH[1]}" "${BASH_REMATCH[3]}" >"$derived"
	printf 'perl\t%s\n' "$derived"
}

cores() {
	local core file
	for core in $(wrap_cores); do
		for file in "$root/puzzle/$puzzle/solution/$core"/main.* "$root/puzzle/$puzzle/core/$core"/*; do
			printf '%s\t%s\n' "$core" "$file"
			[[ $core != bash ]] || derived_perl_core "$file"
		done
	done
}

available() {
	awk -F'\t' -v core="$2" 'NR > 1 && $1 == core && $2 == "yes" { found = 1 } END { exit !found }' "$wrap_root/$1/available.tsv"
}

rejected() {
	[[ -s $wrap_rejected ]] && grep -q -F "$(sha256sum <"$1" | cut -c1-64)" "$wrap_rejected"
}

candidates() {
	local host=$1 core file count=0
	while IFS=$'\t' read -r core file; do
		[[ $core != "$host" && -x $wrap_root/$host/$core.sh ]] && available "$host" "$core" || continue
		count=$((count + 1))
		local out=$work/$host/$count
		mkdir -p "$out"
		"$wrap_root/$host/$core.sh" "$file" "$kind" "$out" || { echo "wrap.sh: $host/$core.sh failed on ${file#"$root"/}" >&2; continue; }
		local candidate
		for candidate in "$out"/*; do
			rejected "$candidate" && continue
			printf '%s\t%s\t%s\n' "$("$tool_dir/count.sh" "$candidate")" "$candidate" "$core:${file#"$root"/}"
		done
	done <"$work/cores"
}

busy() {
	local host=$1 main=$2
	[[ -n $(compgen -G "$state/running/cell.$puzzle.$host.*.json") || -n $(compgen -G "$state/running/review.-.$host.*.json") ]] && return 0
	[[ -e $main ]] || return 1
	[[ -n $(git -C "$root" ls-files -- "$main") ]] || return 0
	! git -C "$root" diff --quiet -- "$main"
}

cores >"$work/cores"
submit_cap_reached=
for host_dir in "$wrap_root"/*/; do
	host=$(basename "$host_dir")
	host_started=$EPOCHSECONDS
	[[ -f $host_dir/available.tsv && ( -z $host_filter || $host == "$host_filter" ) ]] || continue
	main=$root/puzzle/$puzzle/solution/$host/main.$(tsv_field lang "$host" extension)
	if [[ $mode != estimate ]] && busy "$host" "$main"; then
		printf '%s\tdeferred: cell busy\n' "$host"
		deferred=1
		continue
	fi
	ours=$(jq -r '.ours // empty' "$(ensure_status cell "$puzzle" "$host")")
	ours=${ours%.*}
	mapfile -t sorted < <(candidates "$host" | sort -n -k1,1)
	if [[ $mode == estimate ]]; then
		best=-
		((${#sorted[@]})) && best=${sorted[0]%%$'\t'*}
		printf '%s\t%s\t%s\n' "$host" "${ours:--}" "$best"
		continue
	fi
	winner=-
	for line in "${sorted[@]}"; do
		IFS=$'\t' read -r length candidate source <<<"$line"
		[[ -z $ours ]] || ((length < ours)) || break
		if [[ $mode == check ]]; then
			"$root/puzzle/$puzzle/referee/run.sh" "$candidate" --lang "$host" "${local_sets[@]}" >"$candidate.referee" 2>&1 || continue
			winner="$length $source"
			break
		fi
		previous=$work/previous.$host
		rm -f "$previous"
		[[ -e $main ]] && cp "$main" "$previous"
		mkdir -p "$(dirname "$main")"
		cp "$candidate" "$main"
		attempt_started=$EPOCHSECONDS
		submitted=
		output=$(CG_WRAP_SOURCE=$source "$tool_dir/submit.sh" "$main" 2>&1) && submitted=1
		reason=$({ grep -v '^submit.sh: time ' <<<"$output" || true; } | tail -n 1)
		phases=$({ grep '^submit.sh: time ' <<<"$output" || true; } | tail -n 1 | cut -c17-)
		printf '%s\ttry %s %s\t%s in %ss (%s)\n' "$host" "$length" "$source" "${submitted:+submitted}${submitted:-$reason}" \
			"$((EPOCHSECONDS - attempt_started))" "${phases:-no CG call}"
		[[ -z $submitted ]] || { winner="$length $source submitted"; break; }
		if [[ -e $previous ]]; then cp "$previous" "$main"; else rm -f "$main"; fi
		[[ $output == *"local referee fails"* ]] && continue
		[[ $output == *"did not confirm"* ]] && continue
		[[ $output == *"agent submits on"* ]] && submit_cap_reached=1 deferred=1
		winner="$length $source refused: $reason"
		break
	done
	printf '%s\t%s\t%s\t%ss\n' "$host" "${ours:--}" "$winner" "$((EPOCHSECONDS - host_started))"
	[[ -z $submit_cap_reached ]] || break
done
[[ -z $deferred ]] || exit 3
