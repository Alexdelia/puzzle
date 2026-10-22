#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
mapfile -t arguments < <(perl_args)
emit ts "$(ts_exec "$(shell_command perl "${arguments[@]}")")"
emit ts "$(ts_spawn perl "${arguments[@]}" "$core_body")"
