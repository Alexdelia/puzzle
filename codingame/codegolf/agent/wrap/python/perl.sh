#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
mapfile -t arguments < <(perl_args)
emit_system "$(shell_command perl "${arguments[@]}")"
emit_exec perl "${arguments[@]}" "$core_body"
