#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
[[ $core_body == '<?'* && $core_body != *'?>'* ]] || exit 0
case $core_body in
'<?php'[[:space:]]*) core_body=${core_body#<?php} ;;
'<?='*) core_body="echo ${core_body#<?=}" ;;
*) core_body=${core_body#<?} ;;
esac
for options in "-r" "-ddisplay_errors= -r"; do
	read -r -a arguments <<<"$options"
	emit_lua_execute "$(shell_command php "${arguments[@]}")"
	emit_lua_execute "$(shell_command_double_quoted php "${arguments[@]}")"
done
