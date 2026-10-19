#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../cache.sh"

src=${1:?usage: build.sh <file>}
dir=$(cache_dir go "$src")
if [[ ! -x $dir/run ]]; then
	cp "$src" "$dir/main.go"
	go_home=$build_root/go/home
	mkdir -p "$go_home"
	(
		cd "$dir"
		HOME=$go_home GOPATH=$go_home/gopath GOCACHE=$go_home/cache GO111MODULE=off CGO_ENABLED=0 \
			"$CG_GO/bin/go" build -o main main.go
	)
	write_launcher "$dir" /usr/bin/env GOMAXPROCS=1 "$dir/main"
fi
echo "$dir/run"
