#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_lua_execute "$(shell_command /usr/local/bin/ruby -e)"
emit_lua_execute "$(shell_command_double_quoted /usr/local/bin/ruby -e)"
