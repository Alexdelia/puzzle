#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit ts "$(ts_exec "$(shell_command python3 -c)")"
emit ts "$(ts_spawn python3 -c "$core_body")"
