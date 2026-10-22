#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
code=$core_text
if [[ $code == '<?php'[[:space:]]* ]]; then
	code=${code#<?php}
elif [[ $code == '<?'* && $code != '<?='* ]]; then
	code=${code#<?}
else
	exit 0
fi
[[ $code == *'?>'* ]] && exit 0
core_body=$code
emit_system "$(shell_command php -r)"
emit_exec php -r "$code"
