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
emit_clojure php -r "$core_body"
emit_clojure php -ddisplay_errors= -r "$core_body"
