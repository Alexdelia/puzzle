#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
bin=$root/target/release
pinned="env BTK_THINK_MS=200 BTK_PLOTS=0"

if [ "${1:-}" = "--game" ]; then
	seed=$2
	side=$3
	if [ "$side" = 0 ]; then p1=$CAND; p2=$FOE; else p1=$FOE; p2=$CAND; fi
	"$BIN/study" --play --p1 "$p1" --p2 "$p2" --seed "$seed" 2>/dev/null |
		awk -v side="$side" '
			/^seed .* turns / { won = ($0 ~ (side == 0 ? " p1 wins" : " p2 wins")) ? 1 : 0; mine = $(4 + side); yours = $(5 - side) }
			/^dead claims +[0-9]/ { dead[0] = $3; dead[1] = $4 }
			/never on a linked pair/ { orphan[0] = $6; orphan[1] = $7 }
			/^paint sunk in dead claims/ { sunk[0] = $6; sunk[1] = $7 }
			/^cells that paid/ { paying[0] = $4; paying[1] = $5 }
			/^points per paying cell/ { per[0] = $5; per[1] = $6 }
			/^  turns on a path/ { alive[0] = $5; alive[1] = $6 }
			/^  share of paint spent/ { gsub("%", ""); share[0] = $5; share[1] = $6 }
			END { print won, share[side], dead[side], orphan[side], mine, yours, paying[side], per[side], per[1 - side], alive[side], alive[1 - side] }
		'
	exit 0
fi

cand=${1:?usage: yield.sh CANDIDATE_SPEC [FOE_SPEC] [games] [seed]}
foe=${2:-sentinel}
games=${3:-40}
seed=${4:-7000000}

snap=$(mktemp -d "${TMPDIR:-/tmp}/yield.XXXXXX")
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
export CAND FOE
export BIN=$bin

for ((i = 0; i < games; i++)); do
	echo "$((seed + i)) 0"
	echo "$((seed + i)) 1"
done | xargs -P "$(nproc)" -n 2 "$0" --game | awk -v cand="$cand" -v foe="$foe" '
	{
		games++; won += $1; share += $2; dead += $3; orphan += $4; mine += $5; yours += $6
		paying += $7; per += $8; foe_per += $9; alive += $10; foe_alive += $11
	}
	END {
		printf "%-32s %6.1f%% win  %5.1f%% points  paying %5.1f at %5.1f (foe %5.1f)  turns %5.1f (foe %5.1f)\n",
			cand " vs " foe, 100 * won / games, 100 * mine / (mine + yours),
			paying / games, per / games, foe_per / games, alive / games, foe_alive / games
	}
'
