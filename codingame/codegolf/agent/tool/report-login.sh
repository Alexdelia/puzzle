#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
umask 077

usage="usage: report-login.sh [submission-id]; run by the user in a terminal of their own: stores the CG browser session for the report broker, then reads the report of that submission (default: our last one)"
[[ -n ${CG_PYTHON:-} ]] || exec direnv exec "$root" "$0" "$@"
if [[ ! -t 0 ]]; then
	echo "report-login.sh: run it in a terminal of your own, the cookie must never pass through an agent" >&2
	exit 2
fi
echo "Browser logged in on codingame.com: DevTools > Network > any www.codingame.com/services/ request > Copy as cURL."
echo "Paste the value of -b '...' (the Cookie header), input hidden:"
IFS= read -rs cookie
echo

mkdir -p "$report_dir"
chmod 700 "$report_dir"
jar=$(mktemp "$report_dir/cookies.XXXXXX")
printf '# Netscape HTTP Cookie File\n' >"$jar"
IFS=';' read -ra pairs <<<"$cookie"
for pair in "${pairs[@]}"; do
	pair=${pair#"${pair%%[![:space:]]*}"}
	name=${pair%%=*}
	case $name in
	cgSession | rememberMe) printf '.codingame.com\tTRUE\t/\tTRUE\t0\t%s\t%s\n' "$name" "${pair#*=}" >>"$jar" ;;
	esac
done
unset cookie pairs pair
if ! grep -q -P '\t(cgSession|rememberMe)\t' "$jar"; then
	rm -f "$jar"
	echo "report-login.sh: no cgSession or rememberMe cookie in the paste" >&2
	exit 1
fi
mv "$jar" "$report_dir/cookies.txt"
rm -f "$report_dir/expired"
start_report_broker

submission=${1:-$({ grep -o 'submission=[0-9]*' "$state/events.log" || true; } | tail -n 1 | cut -d= -f2)}
if [[ -z $submission ]]; then
	echo "report-login.sh: session stored, no submission in events.log to check it with: run report-login.sh <submission-id> to check" >&2
	exit 1
fi
reply=$("$tool_dir/cg-report.sh" "$submission") || { echo "report-login.sh: broker answered $reply" >&2; exit 1; }
jq -r '"report-login.sh: OK, submission \(.submission): \(.score)%, " + ([.validators[] | if .success then "pass" else "fail" end] | join(" "))' <<<"$reply"
