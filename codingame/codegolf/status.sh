#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh
shopt -s nullglob

bold=1
dim=2
red=31
green=32
yellow=33
cyan=36
column_gap=2
fallback_width=120
close_pace_points=10

use_color=0
[[ -t 1 && -z ${NO_COLOR:-} ]] && use_color=1

screen_width=
[[ -t 1 ]] && screen_width=$(tput cols 2>/dev/null || true)
[[ $screen_width =~ ^[0-9]+$ ]] || screen_width=$fallback_width

paint() {
	local sgr=$1 text=$2
	if ((use_color)) && [[ -n $sgr ]]; then
		printf '\e[%sm%s\e[0m' "$sgr" "$text"
	else
		printf '%s' "$text"
	fi
}

section() {
	paint "$bold" "# $1"
	echo
}

hex_to_sgr() {
	local hex=${1#\#}
	printf '38;2;%d;%d;%d' "0x${hex:0:2}" "0x${hex:2:2}" "0x${hex:4:2}"
}

declare -A lang_icon lang_sgr
load_lang_styles() {
	local slug icon color
	while IFS=$'\t' read -r slug icon color; do
		lang_icon[$slug]=$icon
		[[ -z $color ]] || lang_sgr[$slug]=$(hex_to_sgr "$color")
	done < <(awk -F'\t' '
		NR == 1 { for (i = 1; i <= NF; i++) column[$i] = i; next }
		{ print $1 "\t" $column["icon"] "\t" $column["color"] }
	' "$root/data/lang.tsv")
}
load_lang_styles

is_lang() {
	[[ -n ${lang_icon[$1]+known} ]]
}

lang_label() {
	local slug=$1 width=${2:-0}
	printf '%s %-*s' "$(paint "${lang_sgr[$slug]:-}" "${lang_icon[$slug]:- }")" "$width" "$slug"
}

status_sgr() {
	case $1 in
	ready | done | first) echo "$green" ;;
	active | running) echo "$cyan" ;;
	plateau) echo "$yellow" ;;
	blocked | suspect | rejected | failed) echo "$red" ;;
	*) echo "$dim" ;;
	esac
}

event_sgr() {
	case $1 in
	attention) echo "$yellow" ;;
	review) echo "$cyan" ;;
	*) status_sgr "$1" ;;
	esac
}

progress_ratio() {
	local first=$1 total=$2 sgr=$yellow
	if ((first == 0)); then
		sgr=$dim
	elif ((first >= total)); then
		sgr=$green
	fi
	paint "$sgr" "$(printf '%3d/%-3d' "$first" "$total")"
}

pace_sgr() {
	local used=$1 elapsed=$2
	if ((used >= elapsed)); then
		echo "$red"
	elif ((elapsed - used <= close_pace_points)); then
		echo "$yellow"
	else
		echo "$green"
	fi
}

print_grid() {
	local cell_width=$1
	shift
	local entries=("$@")
	local per_row=$(((screen_width + column_gap) / (cell_width + column_gap)))
	((per_row >= 1)) || per_row=1
	local rows=$(((${#entries[@]} + per_row - 1) / per_row))
	local row column index line separator
	separator=$(printf '%*s' "$column_gap" '')
	for ((row = 0; row < rows; row++)); do
		line=
		for ((column = 0; column < per_row; column++)); do
			index=$((column * rows + row))
			((index < ${#entries[@]})) || continue
			[[ -z $line ]] || line+=$separator
			line+=${entries[index]}
		done
		printf '%s\n' "$line"
	done
}

status_counts() {
	local files=("$1"/*.json)
	if ((${#files[@]} == 0)); then
		paint "$dim" none
		echo
		return
	fi
	local count status line=
	while read -r count status; do
		line+="$(paint "$(status_sgr "$status")" "$status") $count  "
	done < <(jq -r .status "${files[@]}" | sort | uniq -c)
	echo "$line"
}

names_with_status() {
	local status=$1 dir=$2
	local files=("$dir"/*.json)
	((${#files[@]})) || return 0
	jq -r --arg status "$status" 'select(.status == $status) | input_filename' "${files[@]}" |
		xargs -r -n1 basename | sed 's/\.json$//' || true
}

blocked_line() {
	local name labels=()
	while read -r name; do
		labels+=("$(lang_label "$name")")
	done < <(names_with_status blocked "$state/lang")
	while read -r name; do
		labels+=("$name")
	done < <(names_with_status blocked "$state/referee")
	((${#labels[@]})) || return 0
	echo "$(paint "$red" "blocked:")  ${labels[*]}"
}

cell_rows() {
	local key_sed=$1
	shift
	jq -r '[input_filename, .status] | @tsv' "$@" | sed "$key_sed" |
		awk -F'\t' '{ total[$1]++; if ($2 == "first") first[$1]++ } END { for (key in total) printf "%s\t%d\t%d\n", key, first[key], total[key] }' |
		sort
}

lang_table() {
	local slug first total entries=() name_width=12
	while IFS=$'\t' read -r slug first total; do
		entries+=("$(lang_label "$slug" "$name_width") $(progress_ratio "$first" "$total")")
	done < <(cell_rows 's|.*/cell/[^/]*/||; s|\.json||' "$@")
	print_grid $((2 + name_width + 1 + 7)) "${entries[@]}"
}

puzzle_table() {
	local puzzle first total rows=() entries=() name_width=0
	while IFS=$'\t' read -r puzzle first total; do
		rows+=("$puzzle"$'\t'"$first"$'\t'"$total")
		((${#puzzle} <= name_width)) || name_width=${#puzzle}
	done < <(cell_rows 's|.*/cell/||; s|/[^/]*\.json||' "$@")
	local row
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r puzzle first total <<<"$row"
		entries+=("$(printf '%-*s' "$name_width" "$puzzle") $(progress_ratio "$first" "$total")")
	done
	print_grid $((name_width + 1 + 7)) "${entries[@]}"
}

smallest_gaps() {
	local gap cell ours top puzzle lang rows=() puzzle_width=0
	while IFS=$'\t' read -r gap cell ours top; do
		rows+=("$gap"$'\t'"$cell"$'\t'"$ours"$'\t'"$top")
		puzzle=${cell%/*}
		((${#puzzle} <= puzzle_width)) || puzzle_width=${#puzzle}
	done < <(jq -r 'select(.status != "first" and .ours != null and .top != null) | [(.ours - .top), input_filename, .ours, .top] | @tsv' "$@" |
		sed 's|[^\t]*/cell/||; s|\.json||' | sort -n | head -10 || true)
	local row
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r gap cell ours top <<<"$row"
		puzzle=${cell%/*}
		lang=${cell#*/}
		printf '  %s %s %-*s  %s %4s  %s %4s\n' "$(paint "$yellow" "$(printf '+%-4s' "$gap")")" "$(lang_label "$lang" 12)" \
			"$puzzle_width" "$puzzle" "$(paint "$dim" ours)" "$ours" "$(paint "$dim" top)" "$top"
	done
}

task_subject() {
	local kind=$1 puzzle=$2 lang=$3 subject
	subject=$(printf '%-7s' "$kind")
	[[ $puzzle == - ]] || subject+=" $puzzle"
	if [[ $lang != - ]]; then
		subject+=" $(lang_label "$lang")"
	fi
	printf '%s' "$subject"
}

running_minutes() {
	local started
	started=$(date -d "$1" +%s 2>/dev/null) || started=$(date +%s)
	echo $((($(date +%s) - started) / 60))
}

running_tasks() {
	local running id worker started_at kind puzzle lang journal last_line
	for running in "$state"/running/*.json; do
		IFS=$'\t' read -r id worker started_at kind puzzle lang < <(
			jq -r '[.id, .worker, .started_at, .kind, .puzzle, .lang] | map(. // "-") | @tsv' "$running" 2>/dev/null || true
		) || continue
		printf '%s %-3s %4sm  %s  %s\n' "$(paint "$cyan" running)" "w$worker" "$(running_minutes "$started_at")" \
			"$(task_subject "$kind" "$puzzle" "$lang")" "$(paint "$dim" "$id")"
		journal=$state/journal/$id.md
		[[ -s $journal ]] || continue
		last_line=$(grep -v '^[[:space:]]*$' "$journal" | tail -n 1 | cut -c1-150 || true)
		[[ -z $last_line ]] || echo "    $(paint "$dim" "$last_line")"
	done
}

time_and_cost() {
	[[ -s $state/time.log ]] || return 0
	awk -F'\t' 'NR > 1 { wall[$3] += $6; cpu[$3] += $7 + $8; cost[$3] += $11; tasks[$3]++ }
		END { for (k in tasks) printf "  %-8s tasks %4d  wall %7.1f h  cpu %7.1f h  cost $%8.2f\n", k, tasks[k], wall[k] / 3600, cpu[k] / 3600, cost[k] }' "$state/time.log" | sort
}

weekly_usage() {
	local stream modified view
	if IFS=$'\t' read -r stream modified < <(newer_running_stream); then
		view=$(mktemp)
		jq -c 'select(.type == "rate_limit_event") | .rate_limit_info' "$stream" | tail -n 1 |
			jq --argjson at "$modified" '. + {recorded_at: $at}' >"$view"
		usage_file=$view
	fi
	[[ -s $usage_file ]] || return 0
	local used elapsed recorded_at five_hour five_hour_sgr=$green
	read -r used elapsed < <(weekly_pace)
	recorded_at=$(date -d "@$(jq .recorded_at "$usage_file")" +%FT%T)
	echo "  weekly usage $(paint "$(pace_sgr "$used" "$elapsed")" "${used}% vs ${elapsed}%") of week elapsed (stop at used + ${weekly_pace_margin_percent} >= elapsed), $(paint "$dim" "as of $recorded_at")"
	five_hour=$(five_hour_used)
	((five_hour + five_hour_task_reserve_percent < 100)) || five_hour_sgr=$red
	echo "  five-hour usage $(paint "$five_hour_sgr" "${five_hour}%"), resets $(date -d "@$(five_hour_resets_at)" +%T) (claim while used + ${five_hour_task_reserve_percent}% x (running + 1) <= 100%)"
	[[ -z ${view:-} ]] || rm -f "$view"
}

print_event() {
	local line=$1
	local timestamp=${line%%$'\t'*} message=${line#*$'\t'}
	local kind=${message%% *} rest=${message#* }
	if [[ $kind == rejected ]]; then
		local puzzle=${rest%% *} after_puzzle=${rest#* }
		local lang=${after_puzzle%% *} detail=${after_puzzle#* }
		if [[ -n $lang ]] && is_lang "$lang"; then
			rest="$puzzle $(lang_label "$lang") $detail"
		fi
	fi
	echo "$(paint "$dim" "$timestamp") $(paint "$(event_sgr "$kind")" "$kind") $rest"
}

section env
echo "langs:    $(status_counts "$state/lang")"
echo "referees: $(status_counts "$state/referee")"
echo "puzzles:  $(status_counts "$state/puzzle")"
blocked_line

cells=("$state"/cell/*/*.json)
if ((${#cells[@]})); then
	echo
	section "cells first/total per lang"
	lang_table "${cells[@]}"
	echo
	section "cells first/total per puzzle"
	puzzle_table "${cells[@]}"
	echo
	section "smallest gaps not first (ours - top)"
	smallest_gaps "${cells[@]}"
fi

echo
section work
queued=("$state"/queue/*.json)
echo "queued: ${#queued[@]}"
running_tasks
if [[ -e $state/STOP ]]; then
	paint "$bold;$yellow" "STOP set: workers stop after their current task"
	echo
fi

echo
section "time and cost (from agent/state/time.log)"
time_and_cost
weekly_usage

reviews=$(grep -P '\treview ' "$state/events.log" 2>/dev/null | tail -5 || true)
attention=$(grep -P '\t(attention|rejected) ' "$state/events.log" 2>/dev/null | tail -5 || true)
if [[ -n $reviews$attention ]]; then
	echo
	section "needs you"
	while IFS= read -r event_line; do
		[[ -z $event_line ]] || print_event "$event_line"
	done <<<"$attention"$'\n'"$reviews"
fi
