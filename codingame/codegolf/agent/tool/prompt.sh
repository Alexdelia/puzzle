#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/lib.sh"
shopt -s nullglob

running=${1:?usage: prompt.sh <running-file>}
read -r id kind puzzle lang depth task_backend < <(jq -r --arg backend "$backend" '[.id, .kind, .puzzle, .lang, .depth, .backend // $backend] | @tsv' "$running")
journals=("$state"/journal/"$kind.$puzzle.$lang".*.md)
journal_list=
if ((${#journals[@]})); then
	journal_list=$(printf '`%s` ' "${journals[@]#"$root"/}")
fi
cell_json=$([[ $kind == cell ]] && jq -c . "$(ensure_status cell "$puzzle" "$lang")" || echo '{}')
lang_var=${lang//-/_}

declare -A values=(
	[ID]=$id [KIND]=$kind [PUZZLE]=$puzzle [LANG]=$lang [DEPTH]=$depth [NEXT_DEPTH]=$((depth + 1))
	[REASONS]=$(jq -r '.reasons | join("; ")' "$running")
	[TRIGGERS]=$(jq -r 'if .triggers == [] then "none" else .triggers | join(", ") end' "$running")
	[JOURNALS]=${journal_list:-none}
	[WALL_MINUTES]=$(($(task_wall_seconds_of "$task_backend") / 60)) [BUDGET]=$task_budget_usd [MEMORY_MAX]=$task_memory_max [PROBE_CAP]=$([[ $kind == validator ]] && echo "$validator_submits_per_task" || echo "$probe_submits_per_task")
	[PRETTY_ID]=$(tsv_field puzzle "$puzzle" pretty_id) [PUZZLE_KIND]=$(tsv_field puzzle "$puzzle" kind)
	[TITLE]=$(tsv_field puzzle "$puzzle" title)
	[CG_NAME]=$(tsv_field lang "$lang" cg_name) [EXTENSION]=$(tsv_field lang "$lang" extension)
	[CG_VERSION]=$(tsv_field lang "$lang" cg_version) [LANG_VAR]=${lang_var^^}
	[CELL]=$cell_json
	[LANG_CELLS]=$([[ $kind == cell || $kind == review ]] && "$tool_dir/cells.sh" lang "$lang" || true)
	[PUZZLE_CELLS]=$([[ $kind == cell ]] && "$tool_dir/cells.sh" puzzle "$puzzle" || true)
)
text=$(cat "$tool_dir/prompt/common.md" "$tool_dir/prompt/$kind.md")
for key in "${!values[@]}"; do
	text=${text//"{{$key}}"/${values[$key]}}
done
printf '%s\n' "$text"
