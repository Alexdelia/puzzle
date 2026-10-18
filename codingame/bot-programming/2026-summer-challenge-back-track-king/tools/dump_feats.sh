#!/usr/bin/env bash
set -euo pipefail

out=${1:?usage: dump_feats.sh OUT_DIR [bot spec extras]}
extra=${2:-}
root=$(cd "$(dirname "$0")/.." && pwd)
bin=$root/target/release
leaders="ziemekb|Indrill|RamzaBeoulve|Rikus"
mkdir -p "$out"

for log in "$root"/.doc/top_log/*/*/*-*; do
	name=$(basename "$log")
	case "$name" in init|*leaderboard*) continue ;; esac
	first=${name%%-*}
	second=${name#*-}
	side=""
	leader=""
	if echo "$second" | grep -Eq "^($leaders)$"; then side=1; leader=$second; fi
	if echo "$first" | grep -Eq "^($leaders)$"; then side=0; leader=$first; fi
	[ -z "$side" ] && continue
	tag=$(basename "$(dirname "$log")")_$name
	feats=$out/$tag.feats.tsv
	labels=$out/$tag.labels.tsv
	rm -f "$feats"
	"$bin/study" "$log" --against "env BTK_FEATS=$feats BTK_THINK_MS=5000 $extra $bin/augur" \
		--against-side "$side" --against-show 0 --against-timeout 20000 > /dev/null 2>&1 || { echo "failed $tag"; continue; }
	"$bin/study" "$log" --turns 2>/dev/null | awk -v who="$leader" '
		/^== timeline/ { on = 1; next }
		/^== / { on = 0 }
		on && $2 == who {
			for (i = 3; i <= NF; i++) if ($i ~ /^\([0-9]+,[0-9]+\)$/) {
				gsub(/[()]/, "", $i); split($i, c, ","); print $1 "\t" c[1] "\t" c[2]
			}
		}' > "$labels"
	echo "$tag $(wc -l < "$feats") rows, $(wc -l < "$labels") labels"
done
