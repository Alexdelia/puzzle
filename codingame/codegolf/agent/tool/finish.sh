#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

usage="usage: finish.sh <running-file> <claude-json> <time-file> <exit-code>"
running=${1:?$usage}
claude_json=${2:?$usage}
time_file=${3:?$usage}
exit_code=${4:?$usage}

read -r id kind puzzle lang depth task_model task_effort < <(jq -r --arg model "$model" --arg effort "$effort" \
	'[.id, .kind, .puzzle, .lang, .depth, .model // $model, .effort // $effort] | @tsv' "$running")
ran_model=$(jq -r 'select(.type == "system" and .subtype == "init") | .model' "${claude_json%.json}.jsonl" 2>/dev/null | head -n 1 || true)
if [[ -n $ran_model && $ran_model != "$task_model" ]]; then
	task_model=$ran_model task_effort=$effort
fi
outcome=$(jq -r '.outcome // "none"' "$running")
note=$(jq -r '.note // ""' "$running")
read -r wall_s cpu_user_s cpu_sys_s < <(tail -n 1 "$time_file" 2>/dev/null || echo "0 0 0")
read -r tokens_in tokens_out cost_usd turns api_s input_tokens cache_write_tokens cache_read_tokens thinking_tokens \
	permission_denials < <(jq -r '
	[((.usage.input_tokens // 0) + (.usage.cache_creation_input_tokens // 0) + (.usage.cache_read_input_tokens // 0)),
	 (.usage.output_tokens // 0), (.total_cost_usd // 0), (.num_turns // 0), ((.duration_api_ms // 0) / 1000),
	 (.usage.input_tokens // 0), (.usage.cache_creation_input_tokens // 0), (.usage.cache_read_input_tokens // 0),
	 (.usage.output_tokens_details.thinking_tokens // 0), (.permission_denials // [] | length)] | @tsv' "$claude_json" 2>/dev/null ||
	echo "0 0 0 0 0 0 0 0 0 0")
read -r cg_calls cg_test_games cg_submits < <(awk -F'\t' -v id="$id" '
	$2 == id { calls++; if ($3 == "POST" && $4 ~ /\/test-games$/) tests++; if ($3 == "POST" && $4 ~ /\/submissions$/) submits++ }
	END { print calls + 0, tests + 0, submits + 0 }' "$state/cg.log" 2>/dev/null || echo "0 0 0")

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
validator/done)
	set_status done
	commit_scope "feat(golf): validators $puzzle" "puzzle/$puzzle/referee/hidden"
	;;
lang/blocked | referee/blocked | validator/blocked)
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
	git -C "$root" --no-optional-locks status --porcelain=v1 --untracked-files=all -- "$@" | cut -c4- | sed "s|^$prefix||"
}

{ changed_paths knowledge/puzzle | grep -o '^knowledge/puzzle/[^/]*\.md' | sort -u || true; } | while read -r file; do
	commit_scope "doc(golf): knowledge puzzle $(basename "$file" .md)" "$file"
done
{ changed_paths knowledge/lang | grep -o '^knowledge/lang/[^/]*' | sort -u || true; } | while read -r dir; do
	commit_scope "doc(golf): knowledge lang $(basename "$dir")" "$dir"
done

{ changed_paths puzzle | grep -E '^puzzle/[^/]+/referee/bench\.txt$' || true; } | while read -r file; do
	bench_puzzle=$(basename "$(dirname "$(dirname "$file")")")
	if [[ $(jq -r .status "$(ensure_status referee "$bench_puzzle" -)") == done ]] && grep -q '^suite .* pass$' "$root/$file"; then
		commit_scope "chore(golf): bench $bench_puzzle" "$file"
	fi
done

{ changed_paths puzzle | grep -o '^puzzle/[^/]*/probe/' | sort -u || true; } | while read -r dir; do
	probe_puzzle=$(basename "$(dirname "$dir")")
	[[ -n $(tsv_field puzzle "$probe_puzzle" slug) ]] || continue
	commit_scope "chore(golf): probes $probe_puzzle" "$dir"
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
	for file in "$state"/validator/*.json; do
		[[ $(jq -r .status "$file") == done ]] || patterns+=("^puzzle/$(basename "$file" .json)/referee/hidden/")
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
	 | del(.worker_pid)' "$running" >"$state/done/$id.json"
rm -f "$running"
time_row=("$(now)" "$id" "$kind" "$puzzle" "$lang" "$wall_s" "$cpu_user_s" "$cpu_sys_s" "$tokens_in" "$tokens_out" "$cost_usd"
	"$outcome" "$(printf '%s' "$note" | tr '\t\n' '  ')" "$turns" "$api_s" "$input_tokens" "$cache_write_tokens"
	"$cache_read_tokens" "$thinking_tokens" "$permission_denials" "$cg_calls" "$cg_test_games" "$cg_submits" "$task_model"
	"$task_effort")
(
	IFS=$'\t'
	with_lock time_log bash -c 'printf "%s\n" "$1" >>"$2"' _ "${time_row[*]}" "$state/time.log"
)
event "finish $id outcome=$outcome wall_s=$wall_s cost_usd=$cost_usd"
