#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: enqueue.sh lang|referee|validator|wrap|puzzle|cell|review|wrapq <puzzle|-> <lang|-> [--reason text] [--trigger file] [--depth n] [--urgent]; --urgent: before every other task of its kind; wrapq <puzzle> -: a wrap generator job for wrapd (after a task left cores or hand-tuned wrappers in puzzle/<p>/core/<lang>/), not a task"
kind=${1:?$usage}
puzzle=${2:?$usage}
lang=${3:?$usage}
shift 3
reason=manual
trigger=
depth=0
urgent=false
while (($#)); do
	case $1 in
	--reason) reason=$2 ;;
	--trigger) trigger=$2 ;;
	--depth) depth=$2 ;;
	--urgent)
		urgent=true
		shift
		continue
		;;
	*) echo "$usage" >&2 && exit 2 ;;
	esac
	shift 2
done

case $kind in
lang | review | wrap) [[ $puzzle == - && -n $(tsv_field lang "$lang" slug) ]] ;;
referee | validator | puzzle | wrapq) [[ $lang == - && -n $(tsv_field puzzle "$puzzle" slug) ]] ;;
cell) [[ -n $(tsv_field puzzle "$puzzle" slug) && -n $(tsv_field lang "$lang" slug) ]] ;;
*) false ;;
esac || { echo "enqueue.sh: bad target $kind $puzzle $lang" >&2; exit 2; }

if [[ $kind == lang ]]; then
	echo "enqueue.sh: refused lang $lang: toolchains live in agent/tool/lang, read-only in the worker sandbox, the user sets them up"
	exit 0
fi
if [[ $kind == wrapq ]]; then
	wrap_enqueue "$puzzle" "${CG_TASK_ID:+$CG_TASK_ID: }$reason"
	echo "enqueue.sh: queued wrap generator job $puzzle"
	exit 0
fi
if ((depth > propagation_max_depth)); then
	echo "enqueue.sh: refused $kind $puzzle $lang: depth $depth > $propagation_max_depth"
	exit 0
fi

enqueue() {
	local key="$kind/$puzzle/$lang"
	local queued
	queued=$(grep -l -F "\"key\": \"$key\"" "$state"/queue/*.json 2>/dev/null | head -1 || true)
	if [[ -n $queued ]]; then
		json_update "$queued" --arg reason "$reason" --arg trigger "$trigger" --argjson urgent "$urgent" \
			'.reasons += [$reason] | if $trigger != "" then .triggers += [$trigger] else . end | if $urgent then .urgent = true else . end'
		echo "enqueue.sh: merged into $(basename "$queued" .json)"
		return
	fi
	local status_path status
	status_path=$(ensure_status "$kind" "$puzzle" "$lang")
	status=$(jq -r .status "$status_path")
	if [[ $kind == cell && $status == first ]]; then
		echo "enqueue.sh: refused $key: already first"
		return
	fi
	if [[ $kind == cell && $status == plateau && -z $trigger && $reason != plateau-retry && $reason != dropped ]]; then
		echo "enqueue.sh: refused $key: plateau without new trigger"
		return
	fi
	local id="$kind.$puzzle.$lang.$(date +%s%N)"
	jq -n --arg id "$id" --arg key "$key" --arg kind "$kind" --arg puzzle "$puzzle" --arg lang "$lang" \
		--arg reason "$reason" --arg trigger "$trigger" --argjson depth "$depth" --arg queued_at "$(now)" --argjson urgent "$urgent" \
		'{id: $id, key: $key, kind: $kind, puzzle: $puzzle, lang: $lang, reasons: [$reason],
		  triggers: (if $trigger == "" then [] else [$trigger] end), depth: $depth, queued_at: $queued_at, urgent: $urgent}' \
		>"$state/queue/$id.json"
	event "enqueue $id reason=$reason trigger=${trigger:--} depth=$depth"
	echo "enqueue.sh: queued $id"
}

with_lock queue enqueue
