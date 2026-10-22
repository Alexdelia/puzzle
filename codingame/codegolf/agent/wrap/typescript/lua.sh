#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit ts "$(ts_exec "$(shell_command lua -e)")"
emit ts "$(ts_spawn lua -e "$core_body")"
