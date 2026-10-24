#!/usr/bin/env bash
set -euo pipefail

usage="usage: cg.sh GET|POST|PUT|DELETE /path|/services/Service/method [json-body-file]"
method=${1:?$usage}
path=${2:?$usage}
body=${3:-}

api=https://www.codingame.com/api/v1
token_file=${CG_TOKEN_FILE:-$HOME/.config/codingame/token}
tool_dir=$(cd "$(dirname "$0")" && pwd)
lock_dir=$tool_dir/../state/lock
source "$tool_dir/config.sh"
mkdir -p "$lock_dir"

[[ -s $token_file ]] || { echo "cg.sh: missing token $token_file, run the codingame skill device flow" >&2; exit 2; }

headers=$(mktemp)
response=$(mktemp)
trap 'rm -f "$headers" "$headers.curl" "$response"' EXIT

curl_args=(-sS -X "$method" -D "$headers" -o "$response" -w '%{http_code}')
if [[ $path == /services/* ]]; then
	url=https://www.codingame.com$path
else
	url=$api$path
	curl_args+=(-H "Authorization: Bearer $(<"$token_file")")
fi
if [[ -n $body ]]; then
	curl_args+=(-H 'Content-Type: application/json' --data-binary "@$body")
fi

if [[ $path == /services/* ]] && ! grep -q -x -F "${path#/services/}" <(tr ' ' '\n' <<<"$cg_allowed_services"); then
	printf '%s\t%s\t%s\t%s\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${CG_TASK_ID:--}" "$method" "$path" refused >>"$lock_dir/../cg.log"
	printf '%s\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "attention cg.sh refused $method $path for ${CG_TASK_ID:--}: not in cg_allowed_services" >>"$lock_dir/../events.log"
	echo "cg.sh: refused, $path is not an allowed service ($cg_allowed_services): only the documented API (https://www.codingame.com/api/v1/) and the services agent/tool scripts use; other players' solutions are off limits" >&2
	exit 3
fi

exec 9>"$lock_dir/cg.lock"
flock 9

header_value() {
	{ grep -i "^$1:" "$headers" || true; } | tail -1 | cut -d: -f2- | tr -d ' \r'
}

log_failure() {
	local message
	message="$(date -u +%Y-%m-%dT%H:%M:%SZ)	${CG_TASK_ID:--}	$method	${path%%\?*}	$status	$1	$(head -c 400 "$response" 2>/dev/null | tr '\t\n' '  ')"
	printf '%s\n' "$message" >>"$lock_dir/../cg-errors.log"
	echo "cg.sh: $method ${path%%\?*} -> $status ($1): $(head -c 400 "$response" 2>/dev/null | tr '\n' ' ')" >&2
}

network_failures=0
while true; do
	status=$(curl "${curl_args[@]}" "$url" 2>"$headers.curl") || status=000
	printf '%s\t%s\t%s\t%s\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${CG_TASK_ID:--}" "$method" "${path%%\?*}" "$status" \
		>>"$lock_dir/../cg.log"
	if [[ $status == 000 ]]; then
		network_failures=$((network_failures + 1))
		log_failure "curl failed ($network_failures/$cg_network_attempts): $(tr '\n' ' ' <"$headers.curl")"
		((network_failures < cg_network_attempts)) || exit 1
		sleep 10
		continue
	fi
	if [[ $status == 429 ]]; then
		retry_after=$(header_value Retry-After)
		[[ $retry_after =~ ^[0-9]+$ ]] || retry_after=$(jq -r '[.limits[]? | select(.remaining == 0) | .resets_in_seconds] | max // empty' "$response" 2>/dev/null || true)
		[[ $retry_after =~ ^[0-9]+$ ]] || retry_after=10
		log_failure "rate limited, retry in ${retry_after}s"
		flock -u 9
		sleep "$retry_after"
		flock 9
		continue
	fi
	remaining=$(jq -r '.limits.requests.remaining // empty' "$response" 2>/dev/null || true)
	if [[ $remaining =~ ^[0-9]+$ ]] && ((remaining < 3)); then
		sleep "$(jq -r '.limits.requests.resets_in_seconds // 10' "$response")"
	fi
	((status < 400)) || log_failure "error"
	cat "$response"
	((status < 400))
	exit
done
