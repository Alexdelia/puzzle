#!/usr/bin/env bash
set -euo pipefail

readonly TASK_TIMEOUT_SECONDS=900
readonly STOP_TIMEOUT_SECONDS=300
readonly COST_LIMIT_USD=1
readonly POLL_SECONDS=5
readonly HELLO_SUBJECT_PATTERN='^hello: t[0-9]{4}-hello hello$'
readonly HELLO_REQUEST='create one task of kind hello (writes: ["hello.txt"]) whose body says: write hello.txt containing the word hello, commit it with `contest commit --milestone hello`, then finish with `contest task done`'
readonly LONG_REQUEST='create one task of kind long whose body says: run `contest job submit --class batch --ram-mb 100 -- sleep 600`, then wait on the job with `contest job wait <job-id>`, repeating the wait while the job runs'

runtime="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
plugin="$(dirname "$runtime")"
contest_bin="$plugin/bin/contest"

usage() {
	echo "usage: smoke.sh <empty-scratch-dir>" >&2
	exit 2
}

[[ $# -eq 1 && -d $1 ]] || usage
scratch="$(realpath "$1")"
[[ -z "$(ls -A "$scratch")" ]] || usage

repo="$scratch/repo"
contest_dir="$repo/contest"
run="$contest_dir/run"

contest() {
	"$contest_bin" "$@"
}

say() {
	echo "smoke: $*"
}

evidence() {
	say "--- status"
	contest status "$contest_dir" || true
	say "--- journal tail"
	tail -n 30 "$run/journal.md" 2>/dev/null || true
	say "--- events tail"
	tail -n 15 "$run/events.jsonl" 2>/dev/null || true
	say "--- supervisor.log tail"
	tail -n 20 "$run/supervisor.log" 2>/dev/null || true
}

fail() {
	{
		say "FAIL: $*"
		evidence
	} >&2
	exit 1
}

supervisor_running() {
	local status
	status="$(contest status "$contest_dir")"
	grep -Eq '^supervisor: (starting|running|stopping)' <<<"$status"
}

stop_leftover_supervisor() {
	if [[ -d $run ]] && supervisor_running; then
		say "stopping leftover supervisor"
		contest stop "$contest_dir" --reason smoke-exit >/dev/null || true
		timeout "$STOP_TIMEOUT_SECONDS" "$contest_bin" stop "$contest_dir" --wait >/dev/null || true
	fi
}
trap stop_leftover_supervisor EXIT

set_config() {
	local key=$1 value=$2
	sed -i "s|^$key = .*|$key = $value|" "$run/config.toml"
	grep -qx "$key = $value" "$run/config.toml" || fail "config key $key not set"
}

task_dir_of_kind() {
	local dir
	for dir in "$run"/tasks/t[0-9][0-9][0-9][0-9]-"$1"; do
		if [[ -d $dir ]]; then
			echo "$dir"
			return
		fi
	done
}

task_state() {
	cat "$1/meta/state" 2>/dev/null || true
}

wait_for_task_done() {
	local kind=$1 deadline=$((SECONDS + TASK_TIMEOUT_SECONDS)) dir state
	while ((SECONDS < deadline)); do
		dir="$(task_dir_of_kind "$kind")"
		state=""
		if [[ -n $dir ]]; then
			state="$(task_state "$dir")"
		fi
		case $state in
		done) echo "$dir"; return ;;
		failed | cancelled | needs-answer) fail "task $(basename "$dir") ended $state" ;;
		esac
		supervisor_running || fail "supervisor died while waiting for task $kind"
		sleep "$POLL_SECONDS"
	done
	fail "task $kind not done within $TASK_TIMEOUT_SECONDS s"
}

running_job() {
	local record
	for record in "$run"/jobs/j*/job.toml; do
		if [[ -f $record ]] && grep -qx 'state = "Running"' "$record"; then
			echo "$record"
			return
		fi
	done
}

wait_for_running_job() {
	local deadline=$((SECONDS + TASK_TIMEOUT_SECONDS)) job
	while ((SECONDS < deadline)); do
		job="$(running_job)"
		if [[ -n $job ]]; then
			basename "$(dirname "$job")"
			return
		fi
		supervisor_running || fail "supervisor died while waiting for a running job"
		sleep "$POLL_SECONDS"
	done
	fail "no job running within $TASK_TIMEOUT_SECONDS s"
}

wait_for_supervisor_gone() {
	local deadline=$((SECONDS + STOP_TIMEOUT_SECONDS))
	while supervisor_running; do
		((SECONDS < deadline)) || fail "supervisor still alive $STOP_TIMEOUT_SECONDS s after stop"
		sleep 1
	done
}

check_cost() {
	local total
	total="$(awk -F '\t' '{ sum += $8 } END { printf "%.4f", sum }' "$run/ledger.tsv")"
	say "cost: $total USD over $(wc -l <"$run/ledger.tsv") agent runs"
	awk -v total="$total" -v limit="$COST_LIMIT_USD" 'BEGIN { exit !(total < limit) }' ||
		fail "cost $total USD not under $COST_LIMIT_USD USD"
}

say "building contest"
cargo build --release --quiet --manifest-path "$runtime/Cargo.toml"

say "repo $repo"
mkdir -p "$contest_dir"
git -C "$repo" init -q
git -C "$repo" config user.name smoke
git -C "$repo" config user.email smoke@example.invalid
git -C "$repo" config commit.gpgsign false
git -C "$repo" commit -q --allow-empty -m "init: smoke repo"

contest init "$contest_dir" --puzzle smoke --mode past-puzzle --plugin-dir "$plugin"
set_config workers 1
set_config worker_model '"haiku"'
set_config manager_model '"haiku"'
set_config worker_budget_usd 0.3
set_config manager_budget_usd 0.15

say "phase 1: hello task"
contest say "$contest_dir" "$HELLO_REQUEST"
contest start "$contest_dir"
hello_dir="$(wait_for_task_done hello)"
hello_task="$(basename "$hello_dir")"
say "$hello_task done"
history="$(git -C "$repo" log --format='%H %s')"
hello_commit="$(awk -v task="$hello_task" 'NF == 4 && $2 == "hello:" && $3 == task && $4 == "hello" && !found { print $1; found = 1 }' <<<"$history")"
[[ -n $hello_commit ]] || fail "no commit 'hello: $hello_task hello'"
hello_subject="$(git -C "$repo" log -1 --format=%s "$hello_commit")"
grep -Eq "$HELLO_SUBJECT_PATTERN" <<<"$hello_subject" ||
	fail "commit subject is not 'hello: <task> hello'"
hello_files="$(git -C "$repo" show --name-only --format= "$hello_commit")"
grep -qx 'contest/hello.txt' <<<"$hello_files" ||
	fail "commit $hello_commit lacks contest/hello.txt"
say "commit $hello_commit has contest/hello.txt"

say "phase 2: stop during a job"
contest say "$contest_dir" "$LONG_REQUEST"
job="$(wait_for_running_job)"
long_dir="$(task_dir_of_kind long)"
[[ -n $long_dir ]] || fail "job $job running but no long task"
say "$job running for $(basename "$long_dir"), stopping"
contest stop "$contest_dir" --reason smoke
wait_for_supervisor_gone
say "supervisor gone"
long_state="$(task_state "$long_dir")"
[[ $long_state == interrupted || $long_state == waiting ]] ||
	fail "task $(basename "$long_dir") is $long_state, not interrupted or waiting"
[[ -s "$long_dir/notes.md" ]] || fail "$(basename "$long_dir")/notes.md is empty"

check_cost
say "ok"
