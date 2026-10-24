#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
umask 077

service_url=https://www.codingame.com/services/Report/findReportBySubmission
jar=$report_dir/cookies.txt
expired=$report_dir/expired

reply_error() {
	jq -cn --arg error "$1" '{error: $error}'
	exit 0
}

IFS=$' \t' read -r -t 10 submission task || reply_error "expected one line: <submission-id> [task-id]"
[[ $submission =~ ^[0-9]{1,12}$ ]] || reply_error "bad submission id"
[[ ${task:-} =~ ^[A-Za-z0-9._+#-]{1,200}$ ]] || task=-
[[ -s $jar ]] || reply_error "no CG session: the user runs agent/tool/report-login.sh"
[[ ! -e $expired || $jar -nt $expired ]] || reply_error "CG session expired: the user runs agent/tool/report-login.sh"

exec 9>"$report_dir/lock"
flock 9
last_call=$(cat "$report_dir/last_call" 2>/dev/null || echo 0)
((last_call + report_min_interval_seconds <= EPOCHSECONDS)) || sleep $((last_call + report_min_interval_seconds - EPOCHSECONDS))
body=$(mktemp "$report_dir/body.XXXXXX")
trap 'rm -f "$body"' EXIT
status=$(curl -sS --max-time 30 -b "$jar" -c "$jar" -o "$body" -w '%{http_code}' \
	-H 'Content-Type: application/json;charset=UTF-8' -H 'Origin: https://www.codingame.com' \
	--data-raw "[$submission]" "$service_url" 2>/dev/null) || status=000
echo "$EPOCHSECONDS" >"$report_dir/last_call"
printf '%s\t%s\t%s\t%s\t%s\n' "$(now)" "$task" POST /services/Report/findReportBySubmission "$status" >>"$state/cg.log"

if [[ $status == 401 || $status == 403 ]]; then
	touch "$expired"
	event "attention report broker: CG answered $status, session expired: run agent/tool/report-login.sh"
	reply_error "CG session expired: the user runs agent/tool/report-login.sh"
fi
jq -e 'type == "object" and has("submissionId")' "$body" >/dev/null 2>&1 ||
	reply_error "CG answered $status: $(head -c 200 "$body" | tr '\n' ' ')"
jq -e --argjson me "$cg_codingamer_id" '.codingamerId == $me' "$body" >/dev/null ||
	reply_error "submission $submission is not ours"
jq -c '{submission: .submissionId, score, criteria, criteria_score: .criteriaScore, completed: (.completedTime != null),
	validators: [.validators[]? | {name, success, weight: .difficulty}]}' "$body"
