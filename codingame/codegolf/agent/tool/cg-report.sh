#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: cg-report.sh <submission-id>; CG's per-validator report (name, pass/fail, weight) of one of our submissions, fetched by the report broker outside the sandbox"
submission=${1:?$usage}
if [[ ! -S $report_socket ]]; then
	jq -cn '{error: "report broker not running: ./start.sh starts it"}'
	exit 1
fi
reply=$(printf '%s %s\n' "$submission" "${CG_TASK_ID:--}" | socat -t 120 -T 150 - "UNIX-CONNECT:$report_socket") || true
[[ -n $reply ]] || reply=$(jq -cn '{error: "report broker sent nothing"}')
printf '%s\n' "$reply"
jq -e 'has("error") | not' <<<"$reply" >/dev/null 2>&1
