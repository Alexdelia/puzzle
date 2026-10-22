#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: report.sh <outcome> [note]; outcomes: lang ready|blocked, referee done|blocked, validator done|blocked, puzzle done, cell first|plateau|continue, review done|continue, wrap done|blocked"
outcome=${1:?$usage}
note=${2:-}
id=${CG_TASK_ID:?CG_TASK_ID unset: report.sh runs inside a worker task}
running=$state/running/$id.json
[[ -f $running ]] || { echo "report.sh: $id not running" >&2; exit 2; }

kind=$(jq -r .kind "$running")
allowed=(ready blocked)
case $kind in
referee | validator | wrap) allowed=(done blocked) ;;
puzzle) allowed=(done) ;;
cell) allowed=(first plateau continue) ;;
review) allowed=(done continue) ;;
esac
[[ " ${allowed[*]} " == *" $outcome "* ]] || { echo "report.sh: $kind outcome must be one of: ${allowed[*]}" >&2; exit 2; }

json_update "$running" --arg outcome "$outcome" --arg note "$note" '.outcome = $outcome | .note = $note'
event "report $id $outcome ${note:+note=$note}"
echo "report.sh: $id $outcome"
