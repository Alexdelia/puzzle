tool_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=$(cd "$tool_dir/../.." && pwd)
state=$root/agent/state
source "$tool_dir/config.sh"
mkdir -p "$state"/{queue,running,done,journal,lock,leaderboard,lang,referee,puzzle,cell,worker}

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

budget_spent() {
	local since
	since=$(date -u -d "-$cost_window_hours hours" +%Y-%m-%dT%H:%M:%SZ)
	awk -F'\t' -v since="$since" 'NR > 1 && $1 >= since { total += $11 } END { printf "%.4f\n", total + 0 }' "$state/time.log" 2>/dev/null || echo 0
}

budget_reserved() {
	jq -s 'map(.budget_usd // 0) | add // 0' "$state"/running/*.json 2>/dev/null || echo 0
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
