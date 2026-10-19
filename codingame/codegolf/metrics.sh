#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source agent/tool/lib.sh

group=${1:-kind}
case $group in
kind) key='$3' ;;
lang) key='$5' ;;
puzzle) key='$4' ;;
cell) key='$4 "/" $5' ;;
*) echo "usage: metrics.sh [kind|lang|puzzle|cell]" >&2 && exit 2 ;;
esac

awk -F'\t' "NR > 1 { k = $key; n[k]++; cost[k] += \$11; wall[k] += \$6; cpu[k] += \$7 + \$8; turns[k] += \$14; api[k] += \$15
		thinking[k] += \$19; denied[k] += \$20; tests[k] += \$22; submits[k] += \$23 }
	END {
		printf \"%-40s %5s %8s %7s %7s %7s %6s %7s %6s %6s %5s\n\", \"$group\", \"tasks\", \"cost\", \"wall_h\", \"cpu_h\", \"api_h\", \"turns\", \"think_k\", \"denied\", \"tests\", \"subs\"
		for (k in n) printf \"%-40s %5d %8.2f %7.2f %7.2f %7.2f %6.0f %7.0f %6d %6d %5d\n\", k, n[k], cost[k], wall[k] / 3600, cpu[k] / 3600, api[k] / 3600, turns[k], thinking[k] / 1000, denied[k], tests[k], submits[k]
	}" "$state/time.log" | { read -r header; echo "$header"; sort; }
