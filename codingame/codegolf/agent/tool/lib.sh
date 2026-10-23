tool_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=$(cd "$tool_dir/../.." && pwd)
state=$root/agent/state
source "$tool_dir/config.sh"
mkdir -p "$state"/{queue,running,done,journal,lock,leaderboard,lang,referee,validator,puzzle,cell,worker}

now() {
	date -u +%Y-%m-%dT%H:%M:%SZ
}

event() {
	printf '%s\t%s\n' "$(now)" "$*" >>"$state/events.log"
}

langs() {
	tail -n +2 "$root/data/lang.tsv" | cut -f1
}

puzzles() {
	tail -n +2 "$root/data/puzzle.tsv" | cut -f1
}

tsv_field() {
	awk -F'\t' -v key="$2" -v column="$3" '
		NR == 1 { for (i = 1; i <= NF; i++) if ($i == column) wanted = i; next }
		$1 == key { print $wanted }
	' "$root/data/$1.tsv"
}

status_file() {
	local kind=$1 puzzle=$2 lang=$3
	case $kind in
	lang) echo "$state/lang/$lang.json" ;;
	referee) echo "$state/referee/$puzzle.json" ;;
	validator) echo "$state/validator/$puzzle.json" ;;
	puzzle) echo "$state/puzzle/$puzzle.json" ;;
	cell) echo "$state/cell/$puzzle/$lang.json" ;;
	review) echo "$state/review/$lang.json" ;;
	wrap) echo "$state/wrap/$lang.json" ;;
	esac
}

json_update() {
	local file=$1
	shift
	local updated
	updated=$(jq "$@" "$file")
	printf '%s\n' "$updated" >"$file"
}

ensure_status() {
	local file
	file=$(status_file "$@")
	if [[ ! -s $file ]]; then
		mkdir -p "$(dirname "$file")"
		echo '{"status": "todo"}' | jq . >"$file"
	fi
	echo "$file"
}

with_lock() {
	local name=$1
	shift
	(
		flock 8
		"$@"
	) 8>"$state/lock/$name.lock"
}

commit_scope() {
	local message=$1
	shift
	local attempt
	for ((attempt = 1; attempt <= commit_attempts; attempt++)); do
		if (
			flock 8
			cd "$root"
			git add -A -- "$@" || exit 1
			git diff --cached --quiet -- "$@" && exit 0
			git commit -q -m "$message" -- "$@" && event "commit $message"
		) 8>"$state/lock/git.lock"; then
			return 0
		fi
		sleep "$commit_retry_seconds"
	done
	event "attention commit failed after $commit_attempts attempts (git busy?): $message: $*"
}

usage_file=$state/usage.json
week_seconds=$((7 * 24 * 3600))

last_rate_limit_info() {
	{ grep -F '"rate_limit_event"' "$1" || true; } | jq -c 'select(.type == "rate_limit_event") | .rate_limit_info' | tail -n 1
}

record_usage() {
	local info recorded_at=${2:-$(date +%s)} written
	info=$(last_rate_limit_info "$1" 2>/dev/null)
	[[ -n $info ]] || return 0
	written=$(mktemp "$usage_file.XXXXXX")
	jq -n --argjson info "$info" --argjson at "$recorded_at" '$info + {recorded_at: $at}' >"$written"
	mv "$written" "$usage_file"
}

usage_recorded_at() {
	[[ -s $usage_file ]] && jq .recorded_at "$usage_file" || echo 0
}

newer_running_stream() {
	local running stream modified newest streams=()
	newest=$(usage_recorded_at)
	for running in "$state"/running/*.json; do
		stream=$state/worker/${running##*/}
		stream=${stream%.json}.jsonl
		[[ -s $stream ]] && streams+=("$stream")
	done
	((${#streams[@]})) || return 1
	while read -r modified stream; do
		((modified > newest)) || return 1
		if grep -q -m 1 '"rate_limit_event"' "$stream"; then
			printf '%s\t%s\n' "$stream" "$modified"
			return 0
		fi
	done < <(stat -c '%Y %n' "${streams[@]}" | sort -s -k1,1nr)
	return 1
}

sync_usage_from_running() {
	local stream modified
	IFS=$'\t' read -r stream modified < <(newer_running_stream) || return 0
	record_usage "$stream" "$modified"
}

refresh_usage() {
	local age=$((usage_max_age_minutes * 60))
	sync_usage_from_running
	if [[ -s $usage_file ]] && (($(date +%s) - $(jq .recorded_at "$usage_file") < age)); then
		return 0
	fi
	local probe
	probe=$(mktemp)
	(cd "$state" && claude -p ok --model "$usage_probe_model" --output-format stream-json --verbose \
		--max-budget-usd 0.05 --no-session-persistence </dev/null >"$probe" 2>/dev/null) || true
	record_usage "$probe"
	rm -f "$probe"
}

weekly_pace() {
	jq -r --argjson now "$(date +%s)" --argjson week "$week_seconds" '
		.unifiedWindows.seven_day as $window
		| [($window.utilization * 100 | floor), (100 - ($window.resetsAt - $now) * 100 / $week | floor)] | @tsv
	' "$usage_file"
}

over_weekly_pace() {
	[[ -s $usage_file && -z ${CG_FULL_WEEK:-} ]] || return 1
	local used elapsed
	read -r used elapsed < <(weekly_pace)
	((used + weekly_pace_margin_percent >= elapsed))
}

five_hour_used() {
	jq -r --argjson now "$(date +%s)" '
		.unifiedWindows.five_hour as $window
		| if ($window.resetsAt // 0) > $now then ($window.utilization * 100 | floor) else 0 end
	' "$usage_file"
}

five_hour_resets_at() {
	jq -r '.unifiedWindows.five_hour.resetsAt' "$usage_file"
}

five_hour_full() {
	[[ -s $usage_file && -z ${CG_FULL_WINDOW:-} ]] || return 1
	local running_tasks=$1
	(($(five_hour_used) + five_hour_task_reserve_percent * (running_tasks + 1) > five_hour_limit_percent))
}

rejected_until() {
	[[ -s $usage_file ]] || return 1
	jq -e -r --argjson now "$(date +%s)" 'select(.status == "rejected" and .resetsAt > $now) | .resetsAt' "$usage_file"
}

services_query() {
	local body
	body=$(mktemp)
	printf '%s' "$2" >"$body"
	"$tool_dir/cg.sh" POST "/services/$1" "$body"
	rm -f "$body"
}

leaderboard_rows() {
	local leaderboard_id column=$2 filter=$3
	leaderboard_id=$(tsv_field puzzle "$1" leaderboard_id)
	services_query Leaderboards/getFilteredPuzzleLeaderboard \
		"$(jq -n -c --arg id "$leaderboard_id" --arg column "$column" --arg filter "$filter" \
			'[$id, null, "global", {active: true, column: $column, filter: $filter}]')" |
		jq -c '[.users[]? | {pseudo, language: .programmingLanguage, length: (.criteriaScore | floor), score}]'
}

leaderboard_summary() {
	jq -c --arg me "$nickname" '
		. as $rows
		| ($rows | map(select(.score == 100))) as $valid
		| ($valid | min_by(.length)) as $best
		| ($rows | to_entries | map(select(.value.pseudo == $me and .value.score == 100)) | first) as $mine
		| {top: $best.length, top_player: $best.pseudo,
		   top_count: ([$valid[] | select(.length == $best.length)] | length),
		   ours: $mine.value.length,
		   rank: (if $mine then $mine.key + 1 else null end),
		   best_tie_rank: (if $mine then ([$valid[] | select(.length < $mine.value.length)] | length) + 1 else null end),
		   players: ($rows | length)}'
}

leaderboard_table_header() {
	printf 'lang\ttop\ttop_player\ttop_count\tours\trank\tbest_tie_rank\tplayers\tfetched_at\n'
}

leaderboard_table_row() {
	jq -r --arg lang "$1" --arg at "$(now)" \
		'[$lang, .top, .top_player, .top_count, .ours, .rank, .best_tie_rank, .players, $at] | map(. // "-") | @tsv' <<<"$2"
}

store_language_summary() {
	local puzzle=$1 lang=$2 summary=$3 cell previous
	cell=$(ensure_status cell "$puzzle" "$lang")
	previous=$(jq -r .status "$cell")
	json_update "$cell" --argjson summary "$summary" \
		'. + ($summary | {top, top_player, ours, rank, best_tie_rank}) | del(.rank_stale)
		 | if .best_tie_rank == 1 then .status = "first" else . end'
	if [[ $previous == first && $(jq -r .best_tie_rank "$cell") != 1 ]]; then
		json_update "$cell" '.status = "todo"'
		"$tool_dir/enqueue.sh" cell "$puzzle" "$lang" --reason dropped >/dev/null
		event "dropped $puzzle $lang"
	fi
}

store_table_row() {
	local puzzle=$1 lang=$2 summary=$3 table=$state/leaderboard/$1.tsv row
	row=$(leaderboard_table_row "$lang" "$summary")
	with_lock "leaderboard-$puzzle" bash -c '
		table=$1 lang=$2 row=$3 header=$4
		{ [[ -s $table ]] && awk -F"\t" -v lang="$lang" "NR == 1 || \$1 != lang" "$table" || printf "%s\n" "$header"; printf "%s\n" "$row"; } >"$table.tmp"
		mv "$table.tmp" "$table"' _ "$table" "$lang" "$row" "$(leaderboard_table_header)"
}

store_own_rows() {
	local puzzle=$1 rows=$2 skipped=${3:-} lang cell stored current
	while read -r lang; do
		[[ $lang != "$skipped" ]] || continue
		cell=$(ensure_status cell "$puzzle" "$lang")
		current=$(jq -c --arg language "$(tsv_field lang "$lang" cg_name)" \
			'[.[] | select(.language == $language and .score == 100) | .length] | min' <<<"$rows")
		stored=$(jq -c '.ours // null | if type == "number" then floor else . end' "$cell")
		[[ $current != "$stored" ]] || continue
		json_update "$cell" --argjson ours "$current" '.ours = $ours | .rank_stale = true'
		event "own row $puzzle $lang ours $stored -> $current (rank stale until next language fetch)"
	done < <(langs)
}

commit_time_log() {
	commit_scope "chore(golf): time" agent/state/time.log
}

loop_pids() {
	local names=$1
	pgrep -f "^([^ ]*/)?bash ([^ ]*/)?agent/tool/($names)\.sh( |$)" || true
}

wrap_rejected=$state/wrap/rejected.tsv
wrap_lottery=$state/wrap/lottery.tsv

wrap_lottery_round() {
	local hash=$1 puzzle=$2 lang=$3 length=$4 source=$5 row=$6
	if jq -e --argjson length "$length" '.length == $length and .score == 0' <<<"$row" >/dev/null; then
		echo 0
		return
	fi
	mkdir -p "$(dirname "$wrap_lottery")"
	with_lock wrap-lottery bash -c '
		file=$1 hash=$2 rounds=$7
		touch "$file"
		round=$(($(awk -F"\t" -v hash="$hash" "\$1 == hash { print \$6 }" "$file") + 1))
		awk -F"\t" -v hash="$hash" "\$1 != hash" "$file" >"$file.tmp"
		((round < rounds)) && printf "%s\t%s\t%s\t%s\t%s\t%s\n" "$hash" "$3" "$4" "$5" "$6" "$round" >>"$file.tmp"
		mv "$file.tmp" "$file"
		((round < rounds)) && echo "$round" || echo 0' _ "$wrap_lottery" "$hash" "$puzzle" "$lang" "$length" "$source" "$wrap_lottery_rounds"
}

wrap_cores() {
	local generator
	for generator in "$root"/agent/wrap/*/*.sh; do
		[[ -x $generator ]] && basename "$generator" .sh
	done | sort -u
}

wrap_enqueue() {
	local puzzle=$1 reason=$2
	mkdir -p "$state/wrapq"
	with_lock wrapq bash -c '
		file=$1/wrapq/$2.json
		if [[ -s $file ]]; then
			jq --arg reason "$3" --arg now "$4" ".reasons += [\$reason] | .queued_at = \$now" "$file" >"$file.tmp"
		else
			jq -n --arg puzzle "$2" --arg reason "$3" --arg now "$4" "{puzzle: \$puzzle, reasons: [\$reason], queued_at: \$now}" >"$file.tmp"
		fi
		mv "$file.tmp" "$file"' _ "$state" "$puzzle" "$reason" "$(date -u +%Y-%m-%dT%H:%M:%S.%NZ)"
	event "wrap enqueue $puzzle reason=$reason"
}
