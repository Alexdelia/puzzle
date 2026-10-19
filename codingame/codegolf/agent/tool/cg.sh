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
mkdir -p "$lock_dir"

[[ -s $token_file ]] || { echo "cg.sh: missing token $token_file, run the codingame skill device flow" >&2; exit 2; }

headers=$(mktemp)
response=$(mktemp)
trap 'rm -f "$headers" "$response"' EXIT

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

exec 9>"$lock_dir/cg.lock"
flock 9

header_value() {
	grep -i "^$1:" "$headers" | tail -1 | cut -d: -f2- | tr -d ' \r'
}

while true; do
	status=$(curl "${curl_args[@]}" "$url")
	if [[ $status == 429 ]]; then
		retry_after=$(header_value Retry-After)
		sleep "${retry_after:-10}"
		continue
	fi
	remaining=$(jq -r '.limits.requests.remaining // empty' "$response" 2>/dev/null || true)
	if [[ -n $remaining && $remaining -lt 3 ]]; then
		sleep "$(jq -r '.limits.requests.resets_in_seconds' "$response")"
	fi
	cat "$response"
	[[ $status -lt 400 ]]
	exit
done
