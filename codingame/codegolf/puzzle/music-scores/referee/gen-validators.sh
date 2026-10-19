#!/usr/bin/env bash
set -euo pipefail
referee_dir=$(cd "$(dirname "$0")" && pwd)
root=$referee_dir/../../..
export CARGO_TARGET_DIR=$root/agent/state/build/cargo/music-scores
cargo build --release --quiet --manifest-path "$referee_dir/Cargo.toml"
generate() {
	"$CARGO_TARGET_DIR/release/validators" --source "$referee_dir/tests/$1.in" --target "$referee_dir/validators/$2" "${@:3}"
}
mkdir -p "$referee_dir/validators"
generate 01 01 --pad-left 300 --pad-top 40 --pad-bottom 30
generate 04 02 --pad-right 900 --pad-top 100
generate 05 03 --order reverse
generate 06 04 --order shuffle --seed 7 --pad-top 12
generate 07 05 --order reverse --pad-left 33
generate 08 06 --order shuffle --seed 3
generate 09 07 --order shuffle --seed 11 --pad-bottom 50
generate 10 08 --order shuffle --seed 5 --pad-right 17
generate 11 09 --order shuffle --seed 2
generate 12 10 --order shuffle --seed 9 --pad-left 400 --pad-top 30
