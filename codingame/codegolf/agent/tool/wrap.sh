#!/usr/bin/env bash
set -Eeuo pipefail
source "$(dirname "$0")/lib.sh"
trap 'echo "wrap.sh: failed at line $LINENO (exit $?): $BASH_COMMAND" >&2' ERR
shopt -s nullglob

usage="usage: wrap.sh <puzzle> [--host <lang>] [--estimate | --submit]; wraps every core solution of the puzzle (perl, python, bash...) in every host lang of agent/wrap/, keeps per host the shortest candidate under our row that passes the local referee; --estimate: lengths only, no referee; --submit: submit.sh the winners (on STOP: no new attempt, posted ones still judged); exit 3: some host deferred (busy cell, submit cap, STOP, generator error)"
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

own_row_of() {
	jq -c --arg language "$(tsv_field lang "$1" cg_name)" '[.[] | select(.language == $language)] | first' <<<"$2"
}

settle_seconds() {
	awk -F'\t' -v host=" $1" -v floor="$wrap_settle_seconds" -v factor="$wrap_settle_factor" -v cap="$wrap_confirm_seconds" '
		substr($1, length($1) - length(host) + 1) == host && $2 ~ /^judged/ && $3 ~ /^submitted after [0-9]+s/ { split($3, words, " "); if (words[3] + 0 > slowest) slowest = words[3] + 0 }
		END { settle = slowest * factor; settle = settle < floor ? floor : settle; print int(settle > cap ? cap : settle) }' "$state/worker/wrapd.log" 2>/dev/null || echo "$wrap_settle_seconds"
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
		grep -v '^submit.sh: time ' "$work/$host.err" | while IFS= read -r detail; do printf '%s\tdetail %s\n' "$host" "$detail"; done || true
		if [[ $reason == *"POST failed"* || $reason == *"failed at line"* || -z $reason ]]; then
			printf 'retry\t%s\t%s\t%s\t%s\n' "$((index - 1))" "$length" "$source" "${reason:-submit.sh died without a message}" >"$work/$host.result"
		elif [[ $reason == *"agent submits on"* ]]; then
			printf 'cap\t%s\n' "$index" >"$work/$host.result"
		else
			printf 'refused\t%s\t%s\t%s\t%s\n' "$index" "$length" "$source" "$reason" >"$work/$host.result"
		fi
		return
	done <"$work/$host.queue"
	printf 'none\t%s\n' "$index" >"$work/$host.result"
}

submit_pipeline() {
	local -A next_index=() host_started=() host_ours=() local_pid=() pending=() retries=() resubmitted=() settle=()
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
		host_started[$host]=$EPOCHSECONDS host_ours[$host]=$ours next_index[$host]=0 settle[$host]=$(settle_seconds "$host")
		candidates "$host" | sort -n -k1,1 | awk -F'\t' -v ours="$ours" 'ours == "" || $1 < ours + 0' >"$work/$host.queue"
		if [[ ! -s $work/$host.queue ]]; then
			finish_host "$host" -
			continue
		fi
		waiting+=("$host")
	done
	while ((${#waiting[@]} + ${#local_pid[@]} + ${#pending[@]})); do
		if [[ -z $cap && -e $state/STOP ]]; then
			cap=1 deferred=1
			printf 'stop: no new attempt, judging %s posted\n' "${#pending[@]}"
		fi
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
			local outcome index length source submission posted_at candidate reason baseline rows
			IFS=$'\t' read -r outcome index length source submission posted_at candidate <"$work/$host.result"
			next_index[$host]=$index
			case $outcome in
			posted)
				baseline=unknown
				rows=$(leaderboard_rows "$puzzle" KEYWORD "$nickname") && baseline=$(own_row_of "$host" "$rows") || true
				pending[$host]="$length"$'\t'"$source"$'\t'"$posted_at"$'\t'"$candidate"$'\t'"$baseline"
				;;
			none) finish_host "$host" - ;;
			retry)
				IFS=$'\t' read -r _ _ length source reason <"$work/$host.result"
				retries[$host]=$((${retries[$host]:-0} + 1))
				if ((retries[$host] <= wrap_post_attempts)); then
					printf '%s\tretry %s %s\tattempt %s of %s after: %s\n' "$host" "$length" "$source" "${retries[$host]}" "$wrap_post_attempts" "$reason"
					waiting+=("$host")
				else
					finish_host "$host" "$length $source refused: $reason"
				fi
				;;
			error) deferred=1 && finish_host "$host" "- refused: attempt died, see wrapd.log" ;;
			cap) cap=1 deferred=1 && finish_host "$host" - ;;
			refused) IFS=$'\t' read -r _ _ length source reason <"$work/$host.result" && finish_host "$host" "$length $source refused: $reason" ;;
			esac
		done
		if ((${#pending[@]})) && ((EPOCHSECONDS >= next_check)); then
			next_check=$((EPOCHSECONDS + 10))
			local rows row verdict judged moved
			if ! rows=$(leaderboard_rows "$puzzle" KEYWORD "$nickname"); then
				echo "wrap.sh: own rows fetch failed, judging again in 10 s" >&2
				rows=
			fi
			for host in "${!pending[@]}"; do
				IFS=$'\t' read -r length source posted_at candidate baseline <<<"${pending[$host]}"
				[[ -n $rows ]] || continue
				row=$(own_row_of "$host" "$rows")
				if [[ $baseline == unknown ]]; then
					[[ $(jq -r '.length // 0 | floor' <<<"$row") == "$length" ]] && moved=1 || moved=
				else
					[[ $row != "$baseline" ]] && moved=1 || moved=
				fi
				[[ -n $moved ]] || ((EPOCHSECONDS - posted_at >= settle[$host])) || continue
				unset "pending[$host]"
				if [[ -z $moved && ${resubmitted[$host]:-} != "${next_index[$host]}" ]]; then
					if [[ -n $cap ]]; then
						deferred=1
						finish_host "$host" "$length $source unconfirmed: row unchanged after $((EPOCHSECONDS - posted_at))s, no resubmit after stop or cap"
						continue
					fi
					resubmitted[$host]=${next_index[$host]} next_index[$host]=$((next_index[$host] - 1))
					printf '%s\tresubmit %s %s\trow unchanged after %ss on CG (lost leaderboard update or failed validator)\n' "$host" "$length" "$source" "$((EPOCHSECONDS - posted_at))"
					waiting=("$host" "${waiting[@]}")
					continue
				fi
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
			[[ -z $rows ]] || store_own_rows "$puzzle" "$rows"
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
