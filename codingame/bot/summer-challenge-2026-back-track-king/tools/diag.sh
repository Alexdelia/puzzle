#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
bin=$root/target/release
pinned="env BTK_THINK_MS=3000 BTK_PLOTS=0"

if [ "${1:-}" = "--game" ]; then
	seed=$2
	side=$3
	if [ "$side" = 0 ]; then p1=$CAND; p2=$FOE; else p1=$FOE; p2=$CAND; fi
	"$BIN/study" --play --me "$side" --p1 "$p1" --p2 "$p2" --seed "$seed" 2>/dev/null |
		awk -v side="$side" '
			function pair(k) { field[k "_us"] = $(NF - 1); field[k "_foe"] = $NF }
			/^seed .* turns / { won = ($0 ~ (side == 0 ? " p1 wins" : " p2 wins")) ? 1 : 0
				mine = $(4 + side); yours = $(5 - side); turns = $7 }
			/^tracks placed/ { pair("placed") }
			/^paint spent/ { split($(NF-1), a, "/"); split($NF, b, "/")
				field["paint_us"] = a[1]; field["paint_foe"] = b[1] }
			/^tracks lost to ink/ { pair("burnt") }
			/^tracks left at the end/ { pair("left") }
			/^disrupts landed/ { pair("inks") }
			/^regions inked/ { pair("regions") }
			/^income swing won by ink/ { pair("swing") }
			/^cells claimed/ { pair("claims") }
			/^never paid a point/ { pair("dead") }
			/^cells that paid/ { pair("paying") }
			/^points per paying cell/ { pair("perpay") }
			/^  turns on a path/ { pair("alive") }
			/^  pairs a paid turn/ { pair("mult") }
			/^  in a town region/ { pair("safe") }
			END {
				printf "%d %d %d %d", won, mine, yours, turns
				n = split("placed paint burnt left inks regions swing claims dead paying perpay alive mult safe", k, " ")
				for (i = 1; i <= n; i++) printf " %s %s", field[k[i] "_us"], field[k[i] "_foe"]
				printf "\n"
			}
		'
	exit 0
fi

cand=${1:?usage: diag.sh CANDIDATE_SPEC [FOE_SPEC] [games] [seed]}
foe=${2:-sentinel}
games=${3:-40}
seed=${4:-7000000}

snap=$(mktemp -d "${TMPDIR:-/tmp}/diag.XXXXXX")
trap 'rm -rf "$snap"' EXIT

snapshot() {
	local spec=$1 name=$2 path pre
	path=${spec##* }
	pre=${spec% *}
	case $path in
		/*) ;;
		*/*) path=$root/$path ;;
		*) path=$bin/$path ;;
	esac
	cp "$path" "$snap/$name"
	if [ "$pre" = "$spec" ]; then echo "$pinned $snap/$name"; else echo "$pre $snap/$name"; fi
}

CAND=$(snapshot "$cand" cand)
FOE=$(snapshot "$foe" foe)
export CAND FOE BIN=$bin

for ((i = 0; i < games; i++)); do
	echo "$((seed + i)) 0"
	echo "$((seed + i)) 1"
done | xargs -P "$(nproc)" -n 2 "$0" --game | awk -v cand="$cand" -v foe="$foe" '
	{
		games++; won += $1; mine += $2; yours += $3; turns += $4
		for (i = 5; i <= NF; i += 2) { a[i] += $i; b[i] += $(i + 1) }
	}
	END {
		n = split("placed paint burnt left inks regions swing claims dead paying perpay alive mult safe", k, " ")
		printf "%s  vs  %s   %d games\n", cand, foe, games
		printf "  %-18s %8.1f%%   points %8.1f %8.1f (%.1f%%)  turns %.1f\n", "win", 100 * won / games,
			mine / games, yours / games, 100 * mine / (mine + yours), turns / games
		for (i = 1; i <= n; i++) {
			j = 3 + 2 * i
			printf "  %-18s %8.2f %8.2f\n", k[i], a[j] / games, b[j] / games
		}
	}
'
