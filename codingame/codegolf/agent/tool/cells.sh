#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"

usage="usage: cells.sh puzzle <puzzle> | cells.sh lang <lang>; our rows of one puzzle in every lang, or of one lang in every puzzle"
view=${1:?$usage}
target=${2:?$usage}

row() {
	local puzzle=$1 lang=$2 label=$3 file
	file=puzzle/$puzzle/solution/$lang/main.$(tsv_field lang "$lang" extension)
	[[ -f $root/$file ]] || file=-
	jq -r --arg label "$label" --arg file "$file" '
		def place: if .best_tie_rank == 1 and .rank == 1 then "first"
			elif .best_tie_rank == 1 then "tied first"
			elif .rank then "#\(.rank)" else "-" end;
		"| \($label) | \(.ours // "-") | \(.top // "-") | \(.top_player // "-") | \(place) | \(.status) | \($file) |"' \
		"$(ensure_status cell "$puzzle" "$lang")"
}

case $view in
puzzle)
	[[ -n $(tsv_field puzzle "$target" slug) ]] || { echo "cells.sh: unknown puzzle $target" >&2; exit 2; }
	echo "| lang | ours | top | top player | our place | status | file |"
	echo "|---|---|---|---|---|---|---|"
	for lang in $(langs); do row "$target" "$lang" "$lang"; done
	;;
lang)
	[[ -n $(tsv_field lang "$target" slug) ]] || { echo "cells.sh: unknown lang $target" >&2; exit 2; }
	echo "| puzzle | ours | top | top player | our place | status | file |"
	echo "|---|---|---|---|---|---|---|"
	for puzzle in $(puzzles); do row "$puzzle" "$target" "$puzzle"; done
	;;
*) echo "$usage" >&2 && exit 2 ;;
esac
echo
echo "ours, top: bytes; top = best row of the language (ours when we lead); leaderboard synced $(cat "$state/last_sync" 2>/dev/null || echo never), own rows refreshed at each submit"
