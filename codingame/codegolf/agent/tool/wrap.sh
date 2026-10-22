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

main_of() {
	echo "$root/puzzle/$puzzle/solution/$1/main.$(tsv_field lang "$1" extension)"
}

attempt_host() {
	local host=$1 from=$2 main index=0 length candidate source started submission reason phases
	main=$(main_of "$host")
	while IFS=$'\t' read -r length candidate source; do
		index=$((index + 1))
		((index > from)) || continue
		started=$EPOCHSECONDS
		submission=$(CG_WRAP_SOURCE=$source "$tool_dir/submit.sh" --no-wait --candidate "$candidate" "$main" 2>"$work/$host.err") || submission=
		reason=$({ grep -v '^submit.sh: time ' "$work/$host.err" || true; } | tail -n 1)
		phases=$({ grep '^submit.sh: time ' "$work/$host.err" || true; } | tail -n 1 | cut -c17-)
		if [[ -n $submission ]]; then
			printf '%s\tposted %s %s\tsubmission %s in %ss (%s)\n' "$host" "$length" "$source" "$submission" "$((EPOCHSECONDS - started))" "$phases"
			printf 'posted\t%s\t%s\t%s\t%s\t%s\t%s\n' "$index" "$length" "$source" "$submission" "$EPOCHSECONDS" "$candidate" >"$work/$host.result"
			return
		fi
		printf '%s\ttry %s %s\t%s in %ss (%s)\n' "$host" "$length" "$source" "$reason" "$((EPOCHSECONDS - started))" "${phases:-no CG call}"
		[[ $reason == *"local referee fails"* ]] && continue
		if [[ $reason == *"agent submits on"* ]]; then
			printf 'cap\t%s\n' "$index" >"$work/$host.result"
		else
			printf 'refused\t%s\t%s\t%s\t%s\n' "$index" "$length" "$source" "$reason" >"$work/$host.result"
		fi
		return
	done <"$work/$host.queue"
	printf 'none\t%s\n' "$index" >"$work/$host.result"
}

submit_pipeline() {
	local -A next_index=() host_started=() host_ours=() local_pid=() pending=()
	local waiting=() host main ours cap= next_check=0
	finish_host() {
		local host=$1 winner=$2
		printf '%s\t%s\t%s\t%ss\n' "$host" "${host_ours[$host]:--}" "$winner" "$((EPOCHSECONDS - host_started[$host]))"
	}
	for host_dir in "$wrap_root"/*/; do
		host=$(basename "$host_dir")
		[[ -f $host_dir/available.tsv && ( -z $host_filter || $host == "$host_filter" ) ]] || continue
		main=$(main_of "$host")
		if busy "$host" "$main"; then
			printf '%s\tdeferred: cell busy\n' "$host"
			deferred=1
			continue
		fi
		ours=$(jq -r '.ours // empty' "$(ensure_status cell "$puzzle" "$host")")
		ours=${ours%.*}
		host_started[$host]=$EPOCHSECONDS host_ours[$host]=$ours next_index[$host]=0
		candidates "$host" | sort -n -k1,1 | awk -F'\t' -v ours="$ours" 'ours == "" || $1 < ours + 0' >"$work/$host.queue"
		if [[ ! -s $work/$host.queue ]]; then
			finish_host "$host" -
			continue
		fi
		waiting+=("$host")
	done
	while ((${#waiting[@]} + ${#local_pid[@]} + ${#pending[@]})); do
		while [[ -z $cap ]] && ((${#waiting[@]} && ${#local_pid[@]} < wrap_local_jobs && ${#local_pid[@]} + ${#pending[@]} < wrap_in_flight)); do
			host=${waiting[0]}
			waiting=("${waiting[@]:1}")
			rm -f "$work/$host.result"
			attempt_host "$host" "${next_index[$host]}" &
			local_pid[$host]=$!
		done
		if [[ -n $cap ]] && ((${#waiting[@]})); then
			for host in "${waiting[@]}"; do finish_host "$host" -; done
			waiting=()
		fi
		for host in "${!local_pid[@]}"; do
			if [[ ! -e $work/$host.result ]]; then
				kill -0 "${local_pid[$host]}" 2>/dev/null && continue
				[[ -e $work/$host.result ]] || printf 'error\t%s\n' "${next_index[$host]}" >"$work/$host.result"
			fi
			wait "${local_pid[$host]}" || true
			unset "local_pid[$host]"
			local outcome index length source submission posted_at candidate reason
			IFS=$'\t' read -r outcome index length source submission posted_at candidate <"$work/$host.result"
			next_index[$host]=$index
			case $outcome in
			posted) pending[$host]="$length"$'\t'"$source"$'\t'"$posted_at"$'\t'"$candidate" ;;
			none) finish_host "$host" - ;;
			error) deferred=1 && finish_host "$host" "- refused: attempt died, see wrapd.log" ;;
			cap) cap=1 deferred=1 && finish_host "$host" - ;;
			refused) IFS=$'\t' read -r _ _ length source reason <"$work/$host.result" && finish_host "$host" "$length $source refused: $reason" ;;
			esac
		done
		if ((${#pending[@]})) && ((EPOCHSECONDS >= next_check)); then
			next_check=$((EPOCHSECONDS + 10))
			local rows row row_length verdict judged
			rows=$(leaderboard_rows "$puzzle" KEYWORD "$nickname")
			for host in "${!pending[@]}"; do
				IFS=$'\t' read -r length source posted_at candidate <<<"${pending[$host]}"
				row=$(jq -c --arg language "$(tsv_field lang "$host" cg_name)" '[.[] | select(.language == $language)] | first' <<<"$rows")
				row_length=$(jq -r '.length // 1e9 | floor' <<<"$row")
				((row_length <= length || EPOCHSECONDS - posted_at >= wrap_confirm_seconds)) || continue
				unset "pending[$host]"
				judged=$(CG_WRAP_SOURCE=$source "$tool_dir/submit.sh" --judge "$row" --candidate "$candidate" "$(main_of "$host")" 2>&1) && verdict=submitted || verdict=
				[[ -n $verdict ]] || verdict=$({ grep -v '^submit.sh: time ' <<<"$judged" || true; } | tail -n 1)
				printf '%s\tjudged %s %s\t%s after %ss on CG\n' "$host" "$length" "$source" "$verdict" "$((EPOCHSECONDS - posted_at))"
				if [[ $verdict == submitted ]]; then
					finish_host "$host" "$length $source submitted"
				elif [[ $verdict == *superseded* ]]; then
					finish_host "$host" "$length $source refused: superseded"
				else
					waiting+=("$host")
				fi
			done
		fi
		sleep 1
	done
}

cores >"$work/cores"
if [[ $mode == submit ]]; then
	submit_pipeline
	[[ -z $deferred ]] || exit 3
	exit 0
fi
for host_dir in "$wrap_root"/*/; do
	host=$(basename "$host_dir")
	[[ -f $host_dir/available.tsv && ( -z $host_filter || $host == "$host_filter" ) ]] || continue
	main=$(main_of "$host")
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
		"$root/puzzle/$puzzle/referee/run.sh" "$candidate" --lang "$host" "${local_sets[@]}" >"$candidate.referee" 2>&1 || continue
		winner="$length $source"
		break
	done
	printf '%s\t%s\t%s\n' "$host" "${ours:--}" "$winner"
done
[[ -z $deferred ]] || exit 3
