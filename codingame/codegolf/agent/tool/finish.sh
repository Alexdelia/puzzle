#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

usage="usage: finish.sh <running-file> <claude-json> <time-file> <exit-code>"
running=${1:?$usage}
claude_json=${2:?$usage}
time_file=${3:?$usage}
exit_code=${4:?$usage}

read -r id kind puzzle lang depth < <(jq -r '[.id, .kind, .puzzle, .lang, .depth] | @tsv' "$running")
outcome=$(jq -r '.outcome // "none"' "$running")
note=$(jq -r '.note // ""' "$running")
read -r wall_s cpu_user_s cpu_sys_s < <(tail -n 1 "$time_file" 2>/dev/null || echo "0 0 0")
read -r tokens_in tokens_out cost_usd < <(jq -r '
	[((.usage.input_tokens // 0) + (.usage.cache_creation_input_tokens // 0) + (.usage.cache_read_input_tokens // 0)),
	 (.usage.output_tokens // 0), (.total_cost_usd // 0)] | @tsv' "$claude_json" 2>/dev/null || echo "0 0 0")

status_path=$(ensure_status "$kind" "$puzzle" "$lang")
set_status() {
	json_update "$status_path" --arg status "$1" --arg note "$note" '.status = $status | .note = $note'
}
requeue() {
	"$tool_dir/enqueue.sh" "$kind" "$puzzle" "$lang" --reason "$1" --depth "$depth" >/dev/null
}

bench_passes() {
	grep -q '^suite .* pass$' "$root/puzzle/$puzzle/referee/bench.txt" 2>/dev/null
}

case $kind/$outcome in
lang/ready)
	json_update "$status_path" --arg drift "$note" '.status = "ready" | .drift = $drift'
	commit_scope "feat(golf): lang $lang" "agent/tool/lang/$lang"
	;;
referee/done)
	if bench_passes; then
		set_status done
		commit_scope "feat(golf): referee $puzzle" "puzzle/$puzzle/referee"
	else
		event "referee $puzzle reported done without passing bench.txt"
		outcome=gate-failed
		set_status todo
		requeue "bench gate failed"
	fi
	;;
lang/blocked | referee/blocked)
	set_status blocked
	event "attention $kind ${lang/#-/}${puzzle/#-/} blocked: $note"
	;;
puzzle/done)
	set_status done
	;;
cell/first)
	set_status first
	;;
cell/continue)
	set_status todo
	requeue continue
	;;
cell/*)
	json_update "$status_path" '.status = "plateau" | .plateau_count = ((.plateau_count // 0) + 1)'
	requeue plateau-retry
	;;
*)
	set_status todo
	requeue "resume after $outcome (exit $exit_code)"
	;;
esac
if [[ $kind == cell ]]; then
	json_update "$status_path" '.attempts = ((.attempts // 0) + 1)'
fi

prefix=$(git -C "$root" rev-parse --show-prefix)
changed_paths() {
	git -C "$root" status --porcelain=v1 --untracked-files=all -- "$@" | cut -c4- | sed "s|^$prefix||"
}

{ changed_paths knowledge/puzzle | grep -o '^knowledge/puzzle/[^/]*\.md' | sort -u || true; } | while read -r file; do
	commit_scope "doc(golf): knowledge puzzle $(basename "$file" .md)" "$file"
done
{ changed_paths knowledge/lang | grep -o '^knowledge/lang/[^/]*' | sort -u || true; } | while read -r dir; do
	commit_scope "doc(golf): knowledge lang $(basename "$dir")" "$dir"
done

in_progress_pattern() {
	local patterns=('^agent/state/' '^knowledge/' '^puzzle/[^/]*/solution/')
	local file
	for file in "$state"/lang/*.json; do
		[[ $(jq -r .status "$file") == ready ]] || patterns+=("^agent/tool/lang/$(basename "$file" .json)/")
	done
	for file in "$state"/referee/*.json; do
		[[ $(jq -r .status "$file") == done ]] || patterns+=("^puzzle/$(basename "$file" .json)/referee/")
	done
	local IFS='|'
	echo "${patterns[*]}"
}
review=$(changed_paths . | grep -v -E "$(in_progress_pattern)" | tr '\n' ' ' || true)
[[ -z $review ]] || event "review $review"

mkdir -p "$state/done"
jq --arg ended_at "$(now)" --arg outcome "$outcome" --arg exit_code "$exit_code" \
	--argjson wall "$wall_s" --argjson cpu_user "$cpu_user_s" --argjson cpu_sys "$cpu_sys_s" \
	--argjson tokens_in "$tokens_in" --argjson tokens_out "$tokens_out" --argjson cost "$cost_usd" \
	'. + {ended_at: $ended_at, outcome: $outcome, exit_code: ($exit_code | tonumber), wall_s: $wall,
	      cpu_user_s: $cpu_user, cpu_sys_s: $cpu_sys, tokens_in: $tokens_in, tokens_out: $tokens_out, cost_usd: $cost}
	 | del(.budget_usd, .worker_pid)' "$running" >"$state/done/$id.json"
rm -f "$running"
printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$(now)" "$id" "$kind" "$puzzle" "$lang" \
	"$wall_s" "$cpu_user_s" "$cpu_sys_s" "$tokens_in" "$tokens_out" "$cost_usd" "$outcome" "$(printf '%s' "$note" | tr '\t\n' '  ')" \
	>>"$state/time.log"
event "finish $id outcome=$outcome wall_s=$wall_s cost_usd=$cost_usd"
