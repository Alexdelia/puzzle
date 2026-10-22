#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
mapfile -t arguments < <(perl_args)
emit_system "$(shell_command ruby "${arguments[@]}")"
emit_exec ruby "${arguments[@]}" "$core_body"
