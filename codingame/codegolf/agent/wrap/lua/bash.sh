#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_lua_execute "$core_body"
emit_lua_execute "$(shell_command bash -c)"
emit_lua_execute "$(shell_command_double_quoted bash -c)"
