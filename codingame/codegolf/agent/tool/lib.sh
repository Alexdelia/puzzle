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
	(
		flock 8
		cd "$root"
		git add -A -- "$@"
		if git diff --cached --quiet -- "$@"; then
			exit 0
		fi
		git commit -q -m "$message" -- "$@" && event "commit $message"
	) 8>"$state/lock/git.lock"
}

usage_file=$state/usage.json
week_seconds=$((7 * 24 * 3600))

last_rate_limit_info() {
	grep -F '"rate_limit_event"' "$1" | jq -c 'select(.type == "rate_limit_event") | .rate_limit_info' | tail -n 1
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
	[[ -s $usage_file ]] || return 1
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
		| ($rows | to_entries | map(select(.value.pseudo == $me)) | first) as $mine
		| {top: $best.length, top_player: $best.pseudo,
		   top_count: ([$valid[] | select(.length == $best.length)] | length),
		   ours: $mine.value.length,
		   rank: (if $mine then $mine.key + 1 else null end),
		   best_tie_rank: (if $mine then ([$valid[] | select(.length < $mine.value.length)] | length) + 1 else null end),
		   players: ($rows | length)}'
}

commit_time_log() {
	commit_scope "chore(golf): time" agent/state/time.log
}
