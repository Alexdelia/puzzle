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
progress_ratio_width=11

use_color=0
[[ -t 1 && -z ${NO_COLOR:-} ]] && use_color=1

screen_width=
[[ -t 1 ]] && screen_width=$(tput cols 2>/dev/null || true)
[[ $screen_width =~ ^[0-9]+$ ]] || screen_width=$fallback_width

paint() {
	if ((use_color)) && [[ -n $2 ]]; then
		printf -v "$1" '\e[%sm%s\e[0m' "$2" "$3"
	else
		printf -v "$1" '%s' "$3"
	fi
}

section() {
	local heading
	paint heading "$bold" "# $1"
	echo "$heading"
}

hex_to_sgr() {
	local hex=${2#\#}
	printf -v "$1" '38;2;%d;%d;%d' "0x${hex:0:2}" "0x${hex:2:2}" "0x${hex:4:2}"
}

declare -A lang_icon lang_sgr
load_lang_styles() {
	local slug icon color sgr
	while IFS=$'\t' read -r slug icon color; do
		lang_icon[$slug]=$icon
		[[ -z $color ]] || {
			hex_to_sgr sgr "$color"
			lang_sgr[$slug]=$sgr
		}
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
	local slug=$2 width=${3:-0} painted_icon
	paint painted_icon "${lang_sgr[$slug]:-}" "${lang_icon[$slug]:- }"
	printf -v "$1" '%s %-*s' "$painted_icon" "$width" "$slug"
}

status_sgr() {
	case $2 in
	ready | done | first) printf -v "$1" '%s' "$green" ;;
	active | running) printf -v "$1" '%s' "$cyan" ;;
	plateau) printf -v "$1" '%s' "$yellow" ;;
	blocked | suspect | rejected | failed) printf -v "$1" '%s' "$red" ;;
	*) printf -v "$1" '%s' "$dim" ;;
	esac
}

event_sgr() {
	case $2 in
	attention) printf -v "$1" '%s' "$yellow" ;;
	review) printf -v "$1" '%s' "$cyan" ;;
	*) status_sgr "$1" "$2" ;;
	esac
}

progress_ratio() {
	local first=$2 ran=$3 total=$4
	local tail="|$ran/$total" first_sgr="$bold;$green" ran_sgr= painted_first painted_bar painted_ran painted_total
	((first)) || first_sgr=$dim
	((ran)) || ran_sgr=$dim
	paint painted_first "$first_sgr" "$first"
	paint painted_bar "$dim" "|"
	paint painted_ran "$ran_sgr" "$ran"
	paint painted_total "$dim" "/$total"
	printf -v "$1" '%*s%s%*s' $((3 - ${#first})) '' "$painted_first$painted_bar$painted_ran$painted_total" $((progress_ratio_width - 3 - ${#tail})) ''
}

pace_sgr() {
	local used=$2 elapsed=$3
	if ((used >= elapsed)); then
		printf -v "$1" '%s' "$red"
	elif ((elapsed - used <= close_pace_points)); then
		printf -v "$1" '%s' "$yellow"
	else
		printf -v "$1" '%s' "$green"
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
	printf -v separator '%*s' "$column_gap" ''
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
	local label=$1 files=("$2"/*.json)
	printf '%-12s' "$label"
	if ((${#files[@]} == 0)); then
		local none
		paint none "$dim" none
		echo "$none"
		return
	fi
	local count status sgr painted line=
	while read -r count status; do
		status_sgr sgr "$status"
		paint painted "$sgr" "$status"
		line+="$painted $count  "
	done < <(jq -r .status "${files[@]}" | sort | uniq -c)
	echo "$line"
}

names_with_status() {
	local status=$1 dir=$2
	local files=("$dir"/*.json)
	((${#files[@]})) || return 0
	jq -r --arg status "$status" 'select(.status == $status) | input_filename' "${files[@]}" |
		sed 's|.*/||; s/\.json$//'
}

blocked_line() {
	local name label labels=() heading
	while read -r name; do
		lang_label label "$name"
		labels+=("$label")
	done < <(names_with_status blocked "$state/lang")
	while read -r name; do
		labels+=("$name")
	done < <(names_with_status blocked "$state/referee")
	((${#labels[@]})) || return 0
	paint heading "$red" "blocked:"
	echo "$heading  ${labels[*]}"
}

cell_table() {
	local cells=("$state"/cell/*/*.json)
	((${#cells[@]})) || return 0
	jq -r '[input_filename, .status, (.attempts // 0), .ours, .top] | @tsv' "${cells[@]}" | sed 's|^[^\t]*/cell/||; s|\.json\t|\t|'
}

first_total_line() {
	local langs=("$state"/lang/*.json) puzzles=("$state"/puzzle/*.json) first painted_first painted_bar
	first=$(awk -F'\t' '$2 == "first" { first++ } END { print first + 0 }' <<<"$cells")
	paint painted_first "$bold;$green" "$first"
	paint painted_bar "$dim" /
	echo "$painted_first$painted_bar$((${#langs[@]} * ${#puzzles[@]}))"
}

cell_rows() {
	local key_field=$1
	awk -F'\t' -v key_field="$key_field" '
		{ split($1, cell, "/"); key = cell[key_field]; total[key]++ }
		$2 == "first" { first[key]++; next }
		$2 != "todo" || $3 > 0 { ran[key]++ }
		END { for (key in total) printf "%s\t%d\t%d\t%d\n", key, first[key], ran[key], total[key] }
	' <<<"$cells" | sort
}

lang_table() {
	local slug first ran total label ratio entries=() name_width=12
	while IFS=$'\t' read -r slug first ran total; do
		lang_label label "$slug" "$name_width"
		progress_ratio ratio "$first" "$ran" "$total"
		entries+=("$label $ratio")
	done < <(cell_rows 2)
	print_grid $((2 + name_width + 1 + progress_ratio_width)) "${entries[@]}"
}

puzzle_table() {
	local puzzle first ran total ratio rows=() entries=() name_width=0
	while IFS=$'\t' read -r puzzle first ran total; do
		rows+=("$puzzle"$'\t'"$first"$'\t'"$ran"$'\t'"$total")
		((${#puzzle} <= name_width)) || name_width=${#puzzle}
	done < <(cell_rows 1)
	local row entry
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r puzzle first ran total <<<"$row"
		progress_ratio ratio "$first" "$ran" "$total"
		printf -v entry '%-*s %s' "$name_width" "$puzzle" "$ratio"
		entries+=("$entry")
	done
	print_grid $((name_width + 1 + progress_ratio_width)) "${entries[@]}"
}

smallest_gaps() {
	local gap cell ours top puzzle lang rows=() puzzle_width=0
	while IFS=$'\t' read -r gap cell ours top; do
		rows+=("$gap"$'\t'"$cell"$'\t'"$ours"$'\t'"$top")
		puzzle=${cell%/*}
		((${#puzzle} <= puzzle_width)) || puzzle_width=${#puzzle}
	done < <(awk -F'\t' '$2 != "first" && $4 != "" && $5 != "" { print $4 - $5 "\t" $1 "\t" $4 "\t" $5 }' <<<"$cells" | sort -n | head -10 || true)
	local row padded_gap painted_gap label ours_tag top_tag
	paint ours_tag "$dim" ours
	paint top_tag "$dim" top
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r gap cell ours top <<<"$row"
		puzzle=${cell%/*}
		lang=${cell#*/}
		printf -v padded_gap '+%-4s' "$gap"
		paint painted_gap "$yellow" "$padded_gap"
		lang_label label "$lang" 12
		printf '  %s %s %-*s  %s %4s  %s %4s\n' "$painted_gap" "$label" "$puzzle_width" "$puzzle" "$ours_tag" "$ours" "$top_tag" "$top"
	done
}

task_subject() {
	local kind=$2 puzzle=$3 lang=$4 task_text label
	printf -v task_text '%-7s' "$kind"
	[[ $puzzle == - ]] || task_text+=" $puzzle"
	if [[ $lang != - ]]; then
		lang_label label "$lang"
		task_text+=" $label"
	fi
	printf -v "$1" '%s' "$task_text"
}

running_rows() {
	local running_files=("$state"/running/*.json)
	((${#running_files[@]})) || return 0
	jq -n -R -r '
		reduce inputs as $line ({}; .[input_filename] += $line + "\n")
		| .[] | fromjson?
		| [.id, .worker, (try ((now - (.started_at | fromdateiso8601)) / 60 | floor) catch 0), .kind, .puzzle, .lang]
		| map(. // "-") | @tsv
	' "${running_files[@]}" 2>/dev/null || true
}

journal_last_lines() {
	local journals=() id
	for id in "$@"; do
		[[ -s $state/journal/$id.md ]] && journals+=("$state/journal/$id.md")
	done
	((${#journals[@]})) || return 0
	awk '
		FNR == 1 && previous { print previous "\t" substr(last, 1, 150) }
		FNR == 1 { previous = FILENAME; sub(/.*\//, "", previous); sub(/\.md$/, "", previous); last = "" }
		NF { last = $0 }
		END { if (previous) print previous "\t" substr(last, 1, 150) }
	' "${journals[@]}"
}

running_tasks() {
	local rows=() row id worker minutes kind puzzle lang ids=() last_line running_tag subject painted_id painted_line
	declare -A last_lines
	mapfile -t rows < <(running_rows)
	for row in "${rows[@]}"; do
		ids+=("${row%%$'\t'*}")
	done
	while IFS=$'\t' read -r id last_line; do
		last_lines[$id]=$last_line
	done < <(journal_last_lines "${ids[@]}")
	paint running_tag "$cyan" running
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r id worker minutes kind puzzle lang <<<"$row"
		task_subject subject "$kind" "$puzzle" "$lang"
		paint painted_id "$dim" "$id"
		printf '%s %-3s %4sm  %s  %s\n' "$running_tag" "w$worker" "$minutes" "$subject" "$painted_id"
		last_line=${last_lines[$id]:-}
		[[ -z $last_line ]] || {
			paint painted_line "$dim" "$last_line"
			echo "    $painted_line"
		}
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
		last_rate_limit_info "$stream" | jq --argjson at "$modified" '. + {recorded_at: $at}' >"$view"
		usage_file=$view
	fi
	[[ -s $usage_file ]] || return 0
	local used elapsed recorded_at resets_at five_hour five_hour_sgr=$green sgr pace painted_five_hour painted_as_of
	read -r used elapsed < <(weekly_pace)
	printf -v recorded_at '%(%FT%T)T' "$(jq .recorded_at "$usage_file")"
	pace_sgr sgr "$used" "$elapsed"
	paint pace "$sgr" "${used}% vs ${elapsed}%"
	paint painted_as_of "$dim" "as of $recorded_at"
	echo "  weekly usage $pace of week elapsed (stop at used + ${weekly_pace_margin_percent} >= elapsed), $painted_as_of"
	five_hour=$(five_hour_used)
	((five_hour + five_hour_task_reserve_percent <= five_hour_limit_percent)) || five_hour_sgr=$red
	paint painted_five_hour "$five_hour_sgr" "${five_hour}%"
	printf -v resets_at '%(%T)T' "$(five_hour_resets_at)"
	echo "  five-hour usage $painted_five_hour, resets $resets_at (claim while used + ${five_hour_task_reserve_percent}% x (running + 1) <= ${five_hour_limit_percent}%)"
	[[ -z ${view:-} ]] || rm -f "$view"
}

print_event() {
	local line=$1
	local timestamp=${line%%$'\t'*} message=${line#*$'\t'}
	local kind=${message%% *} rest=${message#* }
	local label sgr painted_timestamp painted_kind
	if [[ $kind == rejected ]]; then
		local puzzle=${rest%% *} after_puzzle=${rest#* }
		local lang=${after_puzzle%% *} detail=${after_puzzle#* }
		if [[ -n $lang ]] && is_lang "$lang"; then
			lang_label label "$lang"
			rest="$puzzle $label $detail"
		fi
	fi
	event_sgr sgr "$kind"
	paint painted_timestamp "$dim" "$timestamp"
	paint painted_kind "$sgr" "$kind"
	echo "$painted_timestamp $painted_kind $rest"
}

section env
status_counts langs: "$state/lang"
status_counts referees: "$state/referee"
status_counts validators: "$state/validator"
status_counts puzzles: "$state/puzzle"
blocked_line
cells=$(cell_table)
first_total_line

if [[ -n $cells ]]; then
	echo
	section "cells first|ran/total per lang"
	lang_table
	echo
	section "cells first|ran/total per puzzle"
	puzzle_table
	echo
	section "smallest gaps not first (ours - top)"
	smallest_gaps
fi

echo
section work
queued=("$state"/queue/*.json)
echo "queued: ${#queued[@]}"
running_tasks
if [[ -e $state/STOP ]]; then
	paint stop_notice "$bold;$yellow" "STOP set: workers stop after their current task"
	echo "$stop_notice"
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
