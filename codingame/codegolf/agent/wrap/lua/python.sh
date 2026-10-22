#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_lua_execute "$(shell_command python3 -c)"
emit_lua_execute "$(shell_command_double_quoted python3 -c)"
