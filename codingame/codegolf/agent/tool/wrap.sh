#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

usage="usage: wrap.sh <puzzle> [--host <lang>] [--estimate | --submit]; wraps every core solution of the puzzle (perl, python, bash...) in every host lang of agent/wrap/, keeps per host the shortest candidate under our row that passes the local referee; --estimate: lengths only, no referee; --submit: submit.sh the winners"
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
work=$state/build/wrap/$puzzle
rm -rf "$work"
mkdir -p "$work/derived"
local_sets=(--validators)
[[ -e $root/puzzle/$puzzle/referee/hidden/complete ]] && local_sets=(--hidden)

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
	for core in $(langs); do
		[[ -n $(compgen -G "$wrap_root/*/$core.sh") ]] || continue
		for file in "$root/puzzle/$puzzle/solution/$core"/main.* "$root/puzzle/$puzzle/core/$core"/*; do
			printf '%s\t%s\n' "$core" "$file"
			[[ $core == bash ]] && derived_perl_core "$file"
		done
	done
}

available() {
	awk -F'\t' -v core="$2" 'NR > 1 && $1 == core && $2 == "yes" { found = 1 } END { exit !found }' "$wrap_root/$1/available.tsv"
}

restore() {
	local main=$1
	if [[ -n $(git -C "$root" ls-files -- "$main") ]]; then
		git -C "$root" show "HEAD:./${main#"$root"/}" >"$main"
	else
		rm -f "$main"
	fi
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
			printf '%s\t%s\t%s\n' "$("$tool_dir/count.sh" "$candidate")" "$candidate" "$core:${file#"$root"/}"
		done
	done < <(cores)
}

submit_cap_reached=
for host_dir in "$wrap_root"/*/; do
	host=$(basename "$host_dir")
	[[ -f $host_dir/available.tsv && ( -z $host_filter || $host == "$host_filter" ) ]] || continue
	if [[ -n $(compgen -G "$state/running/cell.$puzzle.$host.*.json") ]]; then
		printf '%s\tskipped: cell running\n' "$host"
		continue
	fi
	ours=$(jq -r '.ours // empty' "$(ensure_status cell "$puzzle" "$host")")
	ours=${ours%.*}
	mapfile -t sorted < <(candidates "$host" | sort -n -k1,1)
	if [[ $mode == estimate ]]; then
		best=${sorted[0]%%$'\t'*}
		[[ -n ${sorted[0]:-} ]] || best=-
		printf '%s\t%s\t%s\n' "$host" "${ours:--}" "$best"
		continue
	fi
	winner=-
	main=$root/puzzle/$puzzle/solution/$host/main.$(tsv_field lang "$host" extension)
	for line in "${sorted[@]}"; do
		IFS=$'\t' read -r length candidate source <<<"$line"
		[[ -z $ours ]] || ((length < ours)) || break
		if [[ $mode == check ]]; then
			"$root/puzzle/$puzzle/referee/run.sh" "$candidate" --lang "$host" "${local_sets[@]}" >"$candidate.referee" 2>&1 || continue
			winner="$length $source"
			break
		fi
		mkdir -p "$(dirname "$main")"
		cp "$candidate" "$main"
		output=$("$tool_dir/submit.sh" "$main" 2>&1) && { winner="$length $source submitted"; break; }
		restore "$main"
		[[ $output == *"local referee fails"* ]] && continue
		[[ $output == *"agent submits on"* ]] && submit_cap_reached=1
		winner="$length $source refused: $(tail -n 1 <<<"$output")"
		break
	done
	printf '%s\t%s\t%s\n' "$host" "${ours:--}" "$winner"
	[[ -z $submit_cap_reached ]] || break
done
