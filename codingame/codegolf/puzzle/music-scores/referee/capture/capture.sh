#!/usr/bin/env bash
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$here/../../../..
tests=$here/../tests
work=$here/work
mkdir -p "$work"
for index in "$@"; do
	name=$(printf '%02d' "$index")
	: >"$work/$name.z"
	chunk=0
	while true; do
		code="import sys,zlib,base64;d=sys.stdin.read();z=base64.b85encode(zlib.compress(d.encode(),9)).decode();sys.stderr.write(f'{len(d)} {len(z)}\n'+z[$chunk*3800:$((chunk + 1))*3800]+'\n')"
		jq -n --arg code "$code" --argjson testcase "$index" \
			'{language: "Python3", code: $code, testcase: $testcase}' >"$work/body.json"
		"$root/agent/tool/cg.sh" POST "/puzzles/music-scores-codesize/test-games" "$work/body.json" | jq -j '.output' >"$work/out"
		read -r declared zlen <"$work/out"
		sed -n 2p "$work/out" | tr -d '\n' >>"$work/$name.z"
		chunk=$((chunk + 1))
		got=$(wc -c <"$work/$name.z")
		echo "test $name chunk $chunk: $got of $zlen"
		((got < zlen)) || break
	done
	"$CG_PYTHON/bin/python3" -I -c "import sys,zlib,base64;sys.stdout.write(zlib.decompress(base64.b85decode(open(sys.argv[1]).read())).decode())" "$work/$name.z" >"$tests/$name.in"
	echo "test $name: $(wc -m <"$tests/$name.in") of $declared"
done
