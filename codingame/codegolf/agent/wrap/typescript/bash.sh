#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit ts "$(ts_exec "$core_body")"
emit ts "$(ts_exec "$(shell_command bash -c)")"
emit ts "$(ts_spawn bash -c "$core_body")"
