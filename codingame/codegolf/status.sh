#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh
shopt -s nullglob

bold=1
dim=2
italic=3
red=31
green=32
yellow=33
magenta=35
cyan=36
column_gap=2
fallback_width=120
close_pace_points=10
usage_bar_width=20
needs_you_per_group=5
wrap_eta_history=10
latest_gains_shown=8

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
	paint "$1" "${lang_sgr[$2]:-}" "${lang_icon[$2]:-?}"
}

puzzle_label() {
	local padded
	printf -v padded '%-*s' "${3:-0}" "$2"
	paint "$1" "${4:+$4;}$italic" "$padded"
}

subject_label() {
	local kind=${2% *} name=${2#* }
	if [[ $kind == lang || $kind == wrap || $kind == review ]] && is_lang "$name"; then
		lang_label "$1" "$name"
	else
		puzzle_label "$1" "$name"
	fi
}

status_sgr() {
	case $2 in
	ready | done | first | submitted) printf -v "$1" '%s' "$green" ;;
	active | running) printf -v "$1" '%s' "$cyan" ;;
	plateau | deferred) printf -v "$1" '%s' "$yellow" ;;
	blocked | suspect | rejected | refused | failed) printf -v "$1" '%s' "$red" ;;
	*) printf -v "$1" '%s' "$dim" ;;
	esac
}

paint_count() {
	local count=$3 digits=$4 sgr=$2 padded
	((count)) || sgr=$dim
	printf -v padded '%*s' "$digits" "$count"
	paint "$1" "$sgr" "$padded"
}

progress_ratio_width() {
	echo $((4 * $1 + 3))
}

progress_ratio() {
	local first=$2 ran=$3 wrapped=$4 total=$5 digits=$6
	local painted_first painted_bar painted_ran painted_wrapped painted_total padded_total
	paint_count painted_first "$bold;$green" "$first" "$digits"
	paint painted_bar "$dim" "|"
	paint_count painted_ran "" "$ran" "$digits"
	paint_count painted_wrapped "" "$wrapped" "$digits"
	printf -v padded_total '/%-*s' "$digits" "$total"
	paint painted_total "$dim" "$padded_total"
	printf -v "$1" '%s' "$painted_first$painted_bar$painted_ran$painted_bar$painted_wrapped$painted_total"
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
		puzzle_label label "$name"
		labels+=("$label")
	done < <(names_with_status blocked "$state/referee")
	((${#labels[@]})) || return 0
	paint heading "$red" "blocked:"
	echo "$heading  ${labels[*]}"
}

cell_table() {
	local cells=("$state"/cell/*/*.json)
	((${#cells[@]})) || return 0
	jq -r '[input_filename, .status, (.attempts // 0), .ours, .top, .wrap_source] | @tsv' "${cells[@]}" | sed 's|^[^\t]*/cell/||; s|\.json\t|\t|'
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
		$2 != "todo" || $3 > 0 { ran[key]++; next }
		$6 != "" { wrapped[key]++ }
		END { for (key in total) printf "%s\t%d\t%d\t%d\t%d\n", key, first[key], ran[key], wrapped[key], total[key] }
	' <<<"$cells" | sort
}

count_table() {
	local key_field=$1 key first ran wrapped total ratio label rows=() entries=() name_width=0 digits=1
	while IFS=$'\t' read -r key first ran wrapped total; do
		rows+=("$key"$'\t'"$first"$'\t'"$ran"$'\t'"$wrapped"$'\t'"$total")
		((${#key} <= name_width)) || name_width=${#key}
		((${#total} <= digits)) || digits=${#total}
	done < <(cell_rows "$key_field")
	local row label_width=$name_width
	[[ $key_field != 2 ]] || label_width=1
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r key first ran wrapped total <<<"$row"
		progress_ratio ratio "$first" "$ran" "$wrapped" "$total" "$digits"
		if [[ $key_field == 2 ]]; then
			lang_label label "$key"
		else
			puzzle_label label "$key" "$name_width"
		fi
		entries+=("$label $ratio")
	done
	print_grid $((label_width + 1 + $(progress_ratio_width "$digits"))) "${entries[@]}"
}

smallest_gaps() {
	local gap cell ours top puzzle lang rows=() puzzle_width=0
	while IFS=$'\t' read -r gap cell ours top; do
		rows+=("$gap"$'\t'"$cell"$'\t'"$ours"$'\t'"$top")
		puzzle=${cell%/*}
		((${#puzzle} <= puzzle_width)) || puzzle_width=${#puzzle}
	done < <(awk -F'\t' '$2 != "first" && $4 != "" && $5 != "" { print $4 - $5 "\t" $1 "\t" $4 "\t" $5 }' <<<"$cells" | sort -n | head -10 || true)
	local row padded_gap painted_gap label painted_puzzle ours_tag top_tag
	paint ours_tag "$dim" ours
	paint top_tag "$dim" top
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r gap cell ours top <<<"$row"
		puzzle=${cell%/*}
		lang=${cell#*/}
		printf -v padded_gap '+%-4s' "$gap"
		paint painted_gap "$yellow" "$padded_gap"
		lang_label label "$lang"
		puzzle_label painted_puzzle "$puzzle" "$puzzle_width"
		printf '  %s %s %s  %s %4s  %s %4s\n' "$painted_gap" "$label" "$painted_puzzle" "$ours_tag" "$ours" "$top_tag" "$top"
	done
}

stuck_rows() {
	local cell_files=("$state"/cell/*/*.json)
	((${#cell_files[@]})) || return 0
	jq -n -r --argjson max_streak "$max_plateau_streak" --arg core_langs "$core_langs" \
		--argjson live_keys "$(jq -s '[.[].key]' "$state"/queue/*.json "$state"/running/*.json </dev/null)" '
		[inputs | {key: (input_filename | split("/") | .[-2:] | join("/") | rtrimstr(".json"))} + .]
		| (group_by(.key | split("/")[0]) | map({(.[0].key | split("/")[0]): ([.[].top // empty | select(. > 0)] | min)}) | add) as $shortest_top
		| .[] | (.key | split("/")) as [$puzzle, $lang]
		| select(.status != "first" and (.plateau_streak // 0) >= $max_streak and (("cell/" + .key) | IN($live_keys[]) | not))
		| [$puzzle, $lang, (.ours // "-"), (.top // "-"), .plateau_streak,
			(if ($lang | IN($core_langs | split(" ") | .[])) or (.top != null and .top == $shortest_top[$puzzle]) then "ladder" else "-" end)]
		| @tsv' "${cell_files[@]}"
}

stuck_since() {
	[[ -s $state/events.log ]] || return 0
	awk -F'\t' '
		{ split($2, word, " ") }
		word[1] == "stuck" { sub(/:$/, "", word[3]); at[word[2] "/" word[3]] = mktime(gensub(/[-T:Z]/, " ", "g", $1), 1) }
		END { for (cell in at) print cell "\t" at[cell] }
	' "$state/events.log"
}

stuck_cells() {
	local rows=() row puzzle lang ours top streak ladder puzzle_width=0 none label painted_puzzle ours_tag top_tag painted_streak painted_ladder age
	declare -A stuck_at
	mapfile -t rows < <(stuck_rows)
	if ((${#rows[@]} == 0)); then
		paint none "$dim" none
		echo "  $none"
		return
	fi
	while IFS=$'\t' read -r row age; do
		stuck_at[$row]=$age
	done < <(stuck_since)
	for row in "${rows[@]}"; do
		puzzle=${row%%$'\t'*}
		((${#puzzle} <= puzzle_width)) || puzzle_width=${#puzzle}
	done
	paint ours_tag "$dim" ours
	paint top_tag "$dim" top
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r puzzle lang ours top streak ladder <<<"$row"
		lang_label label "$lang"
		puzzle_label painted_puzzle "$puzzle" "$puzzle_width"
		paint painted_streak "$yellow" "$(printf '%2s without gain' "$streak")"
		painted_ladder=
		[[ $ladder == - ]] || paint painted_ladder "$cyan" "  ladder skipped"
		age=
		[[ -z ${stuck_at[$puzzle/$lang]:-} ]] || {
			age_text age "${stuck_at[$puzzle/$lang]}"
			paint age "$dim" "  $age"
		}
		printf '  %s %s  %s %4s  %s %4s  %s%s%s\n' "$label" "$painted_puzzle" "$ours_tag" "$ours" "$top_tag" "$top" "$painted_streak" "$painted_ladder" "$age"
	done
}

gain_rows() {
	[[ -s $state/events.log ]] || return 0
	awk -F'\t' -v shown="$latest_gains_shown" '
		FILENAME ~ /wrapd\.log$/ {
			if (match($0, /^[^ ]+ wrapd: ([^ ]+) ([^\t]+)\t[^\t]*\t([0-9]+) ([^:]+):[^ ]+ submitted$/, host))
				wrapped[host[1] "\t" host[2] "\t" host[3]] = host[4]
			next
		}
		{ split($2, word, " ") }
		word[1] != "gain" { next }
		{
			cell = word[2] "\t" word[3]
			new_length = ""; previous = last_length[cell]; rank = ""; tie = ""; source = ""
			for (index_ = 4; index_ in word; index_++) {
				split(word[index_], pair, "=")
				if (pair[1] == "length") new_length = pair[2]
				else if (pair[1] == "previous") previous = pair[2]
				else if (pair[1] == "rank") rank = pair[2]
				else if (pair[1] == "best_tie_rank") tie = pair[2]
				else if (pair[1] == "source") source = pair[2]
			}
			if (source == "") source = wrapped[cell "\t" new_length]
			last_length[cell] = new_length
			gain[++gains] = mktime(gensub(/[-T:Z]/, " ", "g", $1), 1) "\t" cell "\t" (previous == "" ? "-" : previous) "\t" new_length "\t" rank "\t" tie "\t" (source == "" ? "-" : source)
		}
		END { for (index_ = gains; index_ >= 1 && index_ > gains - shown; index_--) print gain[index_] }
	' $([[ -s $state/worker/wrapd.log ]] && echo "$state/worker/wrapd.log") "$state/events.log"
}

rank_text() {
	local rank=$2 tie=$3 text sgr=
	text="#$rank"
	[[ -z $tie || $tie == - || $tie == "$rank" ]] || text+=" (tie #$tie)"
	[[ $rank != 1 && $tie != 1 ]] || sgr="$bold;$green"
	paint "$1" "$sgr" "$text"
}

latest_gains() {
	local rows=() row at puzzle lang previous new_length rank tie top source puzzle_width=0 cell age label painted_puzzle painted_previous painted_top painted_rank wrap_text source_label
	mapfile -t rows < <(gain_rows)
	((${#rows[@]})) || return 0
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r at puzzle lang _ <<<"$row"
		((${#puzzle} <= puzzle_width)) || puzzle_width=${#puzzle}
	done
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r at puzzle lang previous new_length rank tie source <<<"$row"
		cell=$state/cell/$puzzle/$lang.json
		top=-
		[[ ! -s $cell ]] || IFS=$'\t' read -r rank tie top < <(jq -r --arg rank "$rank" --arg tie "$tie" '[.rank // $rank, .best_tie_rank // $tie, .top // "-"] | @tsv' "$cell")
		age_text age "$at"
		paint age "$dim" "$(printf '%-11s' "$age")"
		lang_label label "$lang"
		puzzle_label painted_puzzle "$puzzle" "$puzzle_width"
		paint painted_previous "$dim" "$(printf '%5s' "$previous")"
		rank_text painted_rank "$rank" "$tie"
		wrap_text=
		if [[ $source != - ]]; then
			lang_label source_label "$source"
			paint wrap_text "$dim" "wrap from"
			wrap_text="  $wrap_text $source_label $source"
		fi
		paint painted_top "$dim" "$(printf 'first %5s' "$top")"
		printf '  %s %s %s  %s -> %-5s  %s  %s%s\n' "$age" "$label" "$painted_puzzle" "$painted_previous" "$new_length" "$painted_top" "$painted_rank" "$wrap_text"
	done
}

task_subject() {
	local kind=$2 puzzle=$3 lang=$4 task_text label
	printf -v task_text '%-7s' "$kind"
	if [[ $puzzle != - ]]; then
		puzzle_label label "$puzzle"
		task_text+=" $label"
	fi
	if [[ $lang != - ]]; then
		lang_label label "$lang"
		task_text+=" $label"
	fi
	printf -v "$1" '%s' "$task_text"
}

running_rows() {
	local running_files=("$state"/running/*.json)
	((${#running_files[@]})) || return 0
	jq -n -R -r --arg paid_model "$opencode_model" --arg paid_name "$opencode_model_name" '
		reduce inputs as $line ({}; .[input_filename] += $line + "\n")
		| .[] | fromjson?
		| [.id, .worker, (try ((now - (.started_at | fromdateiso8601)) / 60 | floor) catch 0), .kind, .puzzle, .lang,
			((if .model == $paid_model then $paid_name else (.model // "-" | split("/") | last) end) + " " + (.effort // ""))]
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

model_label() {
	local model=$2 padded sgr=$magenta
	printf -v padded '%-*s' "$3" "$model"
	[[ $model != *-free\ * ]] || sgr=$yellow
	sgr="$dim;$sgr"
	paint "$1" "$sgr" "$padded"
}

running_tasks() {
	local rows=() row id worker minutes kind puzzle lang model ids=() last_line running_tag subject painted_id painted_line painted_model model_width=0
	declare -A last_lines
	mapfile -t rows < <(running_rows)
	for row in "${rows[@]}"; do
		ids+=("${row%%$'\t'*}")
		model=${row##*$'\t'}
		((${#model} <= model_width)) || model_width=${#model}
	done
	while IFS=$'\t' read -r id last_line; do
		last_lines[$id]=$last_line
	done < <(journal_last_lines "${ids[@]}")
	paint running_tag "$cyan" running
	for row in "${rows[@]}"; do
		IFS=$'\t' read -r id worker minutes kind puzzle lang model <<<"$row"
		task_subject subject "$kind" "$puzzle" "$lang"
		paint painted_id "$dim" "$id"
		model_label painted_model "$model" "$model_width"
		printf '%s %-3s %4sm  %s  %s  %s\n' "$running_tag" "w$worker" "$minutes" "$painted_model" "$subject" "$painted_id"
		last_line=${last_lines[$id]:-}
		[[ -z $last_line ]] || {
			paint painted_line "$dim" "$last_line"
			echo "    $painted_line"
		}
	done
}

duration_text() {
	local seconds=$2 text
	((seconds > 0)) || seconds=0
	if ((seconds < 3600)); then
		text=$((seconds / 60))m
	elif ((seconds < 86400)); then
		printf -v text '%dh%02dm' $((seconds / 3600)) $((seconds % 3600 / 60))
	else
		text=$((seconds / 86400))d$((seconds % 86400 / 3600))h
	fi
	printf -v "$1" '%s' "$text"
}

age_text() {
	local duration
	duration_text duration $((EPOCHSECONDS - $2))
	printf -v "$1" '%s ago' "$duration"
}

wrap_schedule() {
	local jobs=("$state"/wrapq/*.json) pids uptime
	pids=$(loop_pids wrapd)
	[[ -s $state/worker/wrapd.log && -n $pids ]] || return 0
	uptime=$(ps -o etimes= -p "${pids//$'\n'/,}" | sort -n | tail -1)
	awk -v now="$EPOCHSECONDS" -v daemon_start=$((EPOCHSECONDS - uptime)) -v slots="$wrap_puzzles" -v history="$wrap_eta_history" '
		function iso_epoch(iso) { return mktime(gensub(/[-T:Z]/, " ", "g", iso), 1) }
		function recent_mean(values, count, taken, sum, index_) {
			for (index_ = count; index_ > count - history && index_ >= 1; index_--) { sum += values[index_]; taken++ }
			return taken ? sum / taken : 0
		}
		FNR == NR {
			if ($2 != "wrapd:") next
			if ($3 == "run" && $5 == "estimated" && iso_epoch($1) >= daemon_start) started[$4] = iso_epoch($1)
			else if ($4 == "finished" && $5 == "exit") { run_seconds[++runs] = $8 + 0; delete started[$3] }
			else if ($3 == "estimated" && $5 == "jobs") estimate_seconds[++estimates] = $7 + 0
			next
		}
		{ split($0, job, "\t"); queued[++jobs] = job[1]; retry_at[jobs] = job[2] }
		END {
			run_cost = recent_mean(run_seconds, runs)
			if (!run_cost) exit
			start_cost = recent_mean(estimate_seconds, estimates)
			for (slot = 1; slot <= slots; slot++) free_at[slot] = now
			slot = 0
			for (puzzle in started) {
				finish = started[puzzle] + run_cost
				if (finish < now) finish = now
				if (++slot <= slots) free_at[slot] = finish
				printf "%s\t%d\trunning\n", puzzle, finish
			}
			for (index_ = 1; index_ <= jobs; index_++) {
				if (queued[index_] in started) continue
				earliest = 1
				for (slot = 2; slot <= slots; slot++) if (free_at[slot] < free_at[earliest]) earliest = slot
				begin = free_at[earliest] > retry_at[index_] ? free_at[earliest] : retry_at[index_]
				free_at[earliest] = begin + start_cost + run_cost
				printf "%s\t%d\tqueued\n", queued[index_], free_at[earliest]
			}
		}
	' "$state/worker/wrapd.log" <(((${#jobs[@]})) && jq -rs --argjson now "$EPOCHSECONDS" '
		sort_by([([(.retry_after // 0) - $now, 0] | max), .queued_at])[]
		| [.puzzle, (.retry_after // 0)] | @tsv' "${jobs[@]}")
}

declare -A wrap_eta
wrap_eta_queued=()
wrap_eta_all=0
load_wrap_eta() {
	local puzzle finish kind
	while IFS=$'\t' read -r puzzle finish kind; do
		wrap_eta[$puzzle]=$finish
		[[ $kind != queued ]] || wrap_eta_queued+=("$puzzle")
		((finish <= wrap_eta_all)) || wrap_eta_all=$finish
	done < <(wrap_schedule)
}

eta_text() {
	local finish=$2 duration
	if ((finish <= EPOCHSECONDS)); then
		printf -v "$1" 'due'
	else
		duration_text duration $((finish - EPOCHSECONDS))
		printf -v "$1" '~%s' "$duration"
	fi
}

wrap_queue_line() {
	local jobs=("$state"/wrapq/*.json) daemon ready retry soonest newest line painted eta
	if [[ -n $(loop_pids wrapd) ]]; then
		paint daemon "$green" running
	else
		paint daemon "$dim" "not running"
	fi
	line="wrapd $daemon  jobs ${#jobs[@]}"
	if ((${#jobs[@]})); then
		IFS=$'\t' read -r ready retry soonest newest < <(jq -rs --argjson now "$(date +%s)" '
			[.[] | .retry_after // 0 | select(. > $now)] as $retry
			| [length - ($retry | length), ($retry | length), (($retry | min // $now) - $now), (sort_by(.queued_at) | reverse | .[:3] | map(.puzzle) | join(" "))]
			| @tsv' "${jobs[@]}")
		paint painted "$green" "$ready ready"
		line+=": $painted"
		if ((retry)); then
			duration_text soonest "$soonest"
			paint painted "$yellow" "$retry retry"
			line+=", $painted (first in $soonest)"
		fi
		paint painted "$dim" "newest:"
		line+="  $painted"
		local puzzle
		for puzzle in $newest; do
			puzzle_label painted "$puzzle" 0 "$dim"
			line+=" $painted"
		done
	fi
	if ((wrap_eta_all)); then
		eta_text eta "$wrap_eta_all"
		printf -v painted '%s at %(%H:%M)T' "$eta" "$wrap_eta_all"
		paint painted "$magenta" "$painted"
		line+="  all done $painted"
	fi
	echo "$line"
}

wrap_runs() {
	[[ -s $state/worker/wrapd.log ]] || return 0
	awk '
		function iso_epoch(iso) { return mktime(gensub(/[-T:Z]/, " ", "g", iso), 1) }
		FNR == NR {
			tab = index($0, "\t")
			message = substr($0, tab + 1)
			if (message ~ /^wrap run /) {
				split(message, word, " ")
				ended = ++end_count[word[3]]
				end_at[word[3], ended] = substr($0, 1, tab - 1)
				end_exit[word[3], ended] = substr(word[5], 6)
			}
			next
		}
		!match($0, /^([^ ]+) wrapd: (.*)$/, line) { next }
		match(line[2], /^run ([^ ]+) estimated gain ([0-9]+)$/, run) {
			runs++
			run_puzzle[runs] = run[1]
			run_start[runs] = line[1]
			run_gain[runs] = run[2]
			last_run[run[1]] = runs
			next
		}
		{
			puzzle = substr(line[2], 1, index(line[2], " ") - 1)
			if (!(puzzle in last_run)) next
			current = last_run[puzzle]
			rest = substr(line[2], length(puzzle) + 2)
			fields = split(rest, field, "\t")
			if (fields >= 2 && field[2] ~ /^deferred: /)
				host_line[current, ++hosts[current]] = field[1] "\t-\t-\t-\tdeferred\t" substr(field[2], 11)
			else if (fields >= 3 && match(field[3], /^([0-9]+) ([^:]+):[^ ]+ (submitted|refused: (.*))$/, winner))
				host_line[current, ++hosts[current]] = field[1] "\t" field[2] "\t" winner[1] "\t" winner[2] "\t" (winner[3] == "submitted" ? "submitted" : "refused") "\t" winner[4]
		}
		END {
			for (current = runs > 3 ? runs - 2 : 1; current <= runs; current++) {
				puzzle = run_puzzle[current]
				exit_code = ""
				for (ended = 1; ended <= end_count[puzzle]; ended++)
					if (end_at[puzzle, ended] >= run_start[current]) { exit_code = end_exit[puzzle, ended]; break }
				print "run\t" puzzle "\t" iso_epoch(run_start[current]) "\t" run_gain[current] "\t" exit_code
				for (host = 1; host <= hosts[current]; host++) print "host\t" host_line[current, host]
			}
		}
	' "$state/events.log" "$state/worker/wrapd.log"
}

wrap_generator() {
	load_wrap_eta
	wrap_queue_line
	local kind first second third fourth fifth sixth label source_label painted_puzzle painted_outcome painted_detail painted_start outcome_sgr age run_state eta
	while IFS=$'\t' read -r kind first second third fourth fifth sixth; do
		if [[ $kind == run ]]; then
			case $fourth in
			'')
				paint run_state "$cyan" running
				[[ -z ${wrap_eta[$first]:-} ]] || {
					eta_text eta "${wrap_eta[$first]}"
					paint eta "$magenta" "$eta"
					run_state+=" $eta"
				}
				;;
			0) paint run_state "$green" "done" ;;
			*) paint run_state "$yellow" "deferred (exit $fourth)" ;;
			esac
			age_text age "$second"
			printf -v painted_start '%(%H:%M)T, %s' "$second" "$age"
			paint painted_start "$dim" "$painted_start"
			puzzle_label painted_puzzle "$first"
			printf '  %s  est gain %s  %s  %s\n' "$painted_puzzle" "$third" "$run_state" "$painted_start"
			continue
		fi
		lang_label label "$first"
		source_label=" "
		[[ $fourth == - ]] || lang_label source_label "$fourth"
		status_sgr outcome_sgr "$fifth"
		paint painted_outcome "$outcome_sgr" "$fifth"
		paint painted_detail "$dim" "${sixth:0:80}"
		printf '    %s %5s -> %-5s from %s  %s%s\n' "$label" "$second" "$third" "$source_label" "$painted_outcome" "${sixth:+ $painted_detail}"
	done < <(wrap_runs)
}

time_and_cost() {
	[[ -s $state/time.log ]] || return 0
	local header total
	printf -v header '  %-10s %6s %8s %8s %9s' kind tasks "wall h" "cpu h" "cost \$"
	paint header "$dim" "$header"
	echo "$header"
	awk -F'\t' 'NR > 1 { wall[$3] += $6; cpu[$3] += $7 + $8; cost[$3] += $11; tasks[$3]++ }
		END { for (k in tasks) printf "%.2f\t  %-10s %6d %8.1f %8.1f %9.2f\n", cost[k], k, tasks[k], wall[k] / 3600, cpu[k] / 3600, cost[k] }' "$state/time.log" |
		sort -t$'\t' -k1,1 -rn | cut -f2-
	total=$(awk -F'\t' 'NR > 1 { wall += $6; cpu += $7 + $8; cost += $11; tasks++ }
		END { printf "  %-10s %6d %8.1f %8.1f %9.2f", "total", tasks, wall / 3600, cpu / 3600, cost }' "$state/time.log")
	paint total "$bold" "$total"
	echo "$total"
}

usage_bar() {
	local percent=$2 sgr=$3 marker_percent=${4:--1} cell char cell_sgr painted drawn=
	local filled=$((percent * usage_bar_width / 100)) marker=-1
	((marker_percent < 0)) || marker=$((marker_percent * usage_bar_width / 100))
	((marker < usage_bar_width)) || marker=$((usage_bar_width - 1))
	for ((cell = 0; cell < usage_bar_width; cell++)); do
		if ((cell == marker)); then
			char='┃' cell_sgr=$bold
		elif ((cell < filled)); then
			char='█' cell_sgr=$sgr
		else
			char='░' cell_sgr=$dim
		fi
		paint painted "$cell_sgr" "$char"
		drawn+=$painted
	done
	printf -v "$1" "▕%s▏" "$drawn"
}

usage_bars() {
	local stream modified view
	if IFS=$'\t' read -r stream modified < <(newer_running_stream); then
		view=$(mktemp)
		last_rate_limit_info "$stream" | jq --argjson at "$modified" '. + {recorded_at: $at}' >"$view"
		usage_file=$view
	fi
	[[ -s $usage_file ]] || return 0
	local used elapsed recorded_at age sgr bar verdict painted_as_of
	read -r used elapsed < <(weekly_pace)
	recorded_at=$(jq .recorded_at "$usage_file")
	duration_text age $(($(date +%s) - recorded_at))
	printf -v painted_as_of '%(%T)T, %s ago' "$recorded_at" "$age"
	paint painted_as_of "$dim" "as of $painted_as_of"
	pace_sgr sgr "$used" "$elapsed"
	usage_bar bar "$used" "$sgr" "$elapsed"
	if ((used + weekly_pace_margin_percent >= elapsed)); then
		paint verdict "$red" "over pace"
		every_task_on_claude || paint verdict "$red" "over pace, $claude_kinds held"
	else
		paint verdict "$sgr" "headroom $((elapsed - used - weekly_pace_margin_percent)) pts"
	fi
	echo "weekly  $bar ${used}% used / ${elapsed}% elapsed  $verdict  $painted_as_of"
	local five_hour resets_at resets_in claude_running blocked_text="claims blocked"
	five_hour=$(five_hour_used)
	resets_at=$(five_hour_resets_at)
	claude_running=$(jq -s --arg backend "$backend" '[.[] | select((.backend // $backend) == "claude")] | length' "$state"/running/*.json </dev/null)
	every_task_on_claude || blocked_text="$claude_kinds held"
	sgr=$green
	verdict=
	if ((five_hour + five_hour_task_reserve_percent * (claude_running + 1) > five_hour_limit_percent)); then
		sgr=$red
		paint verdict "$red" "  $blocked_text"
	fi
	usage_bar bar "$five_hour" "$sgr"
	if ((resets_at > $(date +%s))); then
		duration_text resets_in $((resets_at - $(date +%s)))
		printf -v resets_at '%(%H:%M)T' "$resets_at"
		echo "5h      $bar ${five_hour}%  resets in $resets_in at $resets_at$verdict"
	else
		echo "5h      $bar ${five_hour}%  window reset$verdict"
	fi
	[[ -z ${view:-} ]] || rm -f "$view"
}

attention_groups() {
	[[ -s $state/events.log ]] || return 0
	awk -F'\t' '
		function remember(group, key, cell, id) {
			id = group SUBSEP key
			if (!(id in count)) order[++groups] = id
			count[id]++
			last_at[id] = $1
			last_message[id] = $2
			group_of[id] = group
			key_of[id] = key
			cell_of[id] = cell
		}
		{ split($2, word, " ") }
		word[1] == "gain" { gained[word[2] " " word[3]] = $1; next }
		word[1] == "rejected" { remember("rejected", word[2] " " word[3], word[2] " " word[3]); next }
		word[1] == "attention" && word[2] == "wrap" && word[3] == "rejected" { remember("rejected", "wrap " word[4] " " word[5], word[4] " " word[5]); next }
		word[1] == "attention" && word[4] == "blocked:" { remember("blocked", word[2] " " word[3], ""); next }
		word[1] == "attention" && word[2] == "weekly" { remember("stop", "weekly", ""); next }
		word[1] == "attention" { remember("attention", $2, ""); next }
		word[1] == "review" { sub(/ +$/, "", $2); remember("review", $2, ""); next }
		END {
			for (index_ = 1; index_ <= groups; index_++) {
				id = order[index_]
				if (cell_of[id] != "" && gained[cell_of[id]] > last_at[id]) continue
				print group_of[id] "\t" key_of[id] "\t" count[id] "\t" mktime(gensub(/[-T:Z]/, " ", "g", last_at[id]), 1) "\t" last_message[id]
			}
		}
	' "$state/events.log" | sort -t$'\t' -k4,4rn
}

declare -A blocked_subjects dirty_committed_at
load_open_state() {
	local path kind committed_at status_files=("$state"/{lang,referee,validator,puzzle,wrap}/*.json) dirty=()
	((${#status_files[@]})) && while IFS= read -r path; do
		kind=${path%/*}
		blocked_subjects["${kind##*/} $(basename "$path" .json)"]=1
	done < <(jq -r 'select(.status == "blocked") | input_filename' "${status_files[@]}")
	mapfile -t dirty < <(git diff --name-only --relative HEAD -- . 2>/dev/null; git ls-files --others --exclude-standard -- . 2>/dev/null)
	((${#dirty[@]})) || return 0
	for path in "${dirty[@]}"; do
		dirty_committed_at[$path]=0
	done
	while IFS=$'\t' read -r path committed_at; do
		dirty_committed_at[$path]=$committed_at
	done < <(git log --format=$'\x01%ct' --name-only --relative -- "${dirty[@]}" 2>/dev/null |
		awk '/^\x01/ { committed_at = substr($0, 2); next } NF && !seen[$0]++ { print $0 "\t" committed_at }')
}

review_open() {
	local reviewed_at=$1 file
	for file in $2; do
		[[ -n ${dirty_committed_at[$file]+dirty} ]] || continue
		((dirty_committed_at[$file] < reviewed_at)) && return 0
	done
	return 1
}

still_open() {
	local group=$1 key=$2
	case $group in
	blocked) [[ -n ${blocked_subjects[$key]+blocked} ]] ;;
	stop) [[ -e $state/STOP ]] ;;
	review) review_open "$3" "${key#review }" ;;
	*) true ;;
	esac
}

rejection_text() {
	local key=$2 message=$3 puzzle lang length row cg label painted_puzzle source="" wrap=""
	read -r puzzle lang <<<"${key#wrap }"
	[[ $key != wrap\ * ]] || wrap="wrap "
	[[ $message =~ length=([0-9]+) ]] && length=${BASH_REMATCH[1]}
	if [[ $message =~ source=([^:]+): ]]; then
		lang_label source "${BASH_REMATCH[1]}"
		source=" from $source"
	fi
	row=${message#*row=}
	cg="no row"
	if [[ $row =~ \"length\":([0-9]+).*\"score\":([0-9.]+) ]]; then
		printf -v cg '%s %.1f' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
		cg="${cg%.0}%"
	fi
	lang_label label "$lang"
	puzzle_label painted_puzzle "$puzzle"
	printf -v "$1" '%s%s %s %s -> CG %s%s' "$wrap" "$label" "$painted_puzzle" "${length:-?}" "$cg" "$source"
}

needs_you() {
	local rows group key count last_at message
	rows=$(attention_groups)
	[[ -n $rows ]] || return 0
	load_open_state
	declare -A shown
	local -a lines=()
	while IFS=$'\t' read -r group key count last_at message; do
		still_open "$group" "$key" "$last_at" || continue
		((${shown[$group]:-0} < needs_you_per_group)) || continue
		shown[$group]=$((${shown[$group]:-0} + 1))
		lines+=("$group"$'\t'"$key"$'\t'"$count"$'\t'"$last_at"$'\t'"$message")
	done <<<"$rows"
	((${#lines[@]})) || return 0
	echo
	section "needs you"
	local wanted heading sgr age times text label prefix_width=20 line
	for wanted in stop blocked attention rejected review; do
		((${shown[$wanted]:-0})) || continue
		case $wanted in
		rejected) sgr=$red ;;
		review) sgr=$cyan ;;
		*) sgr=$yellow ;;
		esac
		paint heading "$sgr" "$wanted"
		echo "$heading"
		for line in "${lines[@]}"; do
			IFS=$'\t' read -r group key count last_at message <<<"$line"
			[[ $group == "$wanted" ]] || continue
			age_text age "$last_at"
			times=
			((count < 2)) || times="x$count"
			case $group in
			rejected) rejection_text text "$key" "$message" ;;
			blocked)
				subject_label label "$key"
				text=${message#*blocked: }
				text="${key% *} $label: ${text:0:$((screen_width - prefix_width - ${#key} - 2))}"
				;;
			review) text=${key#review } && text=${text:0:$((screen_width - prefix_width))} ;;
			*) text=${message#attention } && text=${text:0:$((screen_width - prefix_width))} ;;
			esac
			paint age "$dim" "$(printf '%-11s' "$age")"
			printf '  %s %-5s %s\n' "$age" "$times" "$text"
		done
	done
}

section env
status_counts langs: "$state/lang"
status_counts referees: "$state/referee"
status_counts validators: "$state/validator"
status_counts puzzles: "$state/puzzle"
status_counts wraps: "$state/wrap"
blocked_line
cells=$(cell_table)
first_total_line

if [[ -n $cells ]]; then
	echo
	section "cells first|ran|wrap/total per lang"
	count_table 2
	echo
	section "cells first|ran|wrap/total per puzzle"
	count_table 1
	echo
	section "smallest gaps not first (ours - top)"
	smallest_gaps
	echo
	section "stuck cells (no gain in $max_plateau_streak tasks in a row, not queued)"
	stuck_cells
fi

echo
section "latest gains (previous -> new, first length, current rank)"
latest_gains

echo
section work
queued=("$state"/queue/*.json)
echo "queued: ${#queued[@]}"
running_tasks
if [[ -e $state/STOP ]]; then
	paint stop_notice "$bold;$yellow" "STOP set: workers stop after their current task"
	echo "$stop_notice"
fi
usage_bars

echo
section "wrap generator"
wrap_generator

echo
section "time and cost (from agent/state/time.log)"
time_and_cost

needs_you
