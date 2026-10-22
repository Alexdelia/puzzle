#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
core_body=${core_body%\?>}
case $core_body in
'<?php'*) core_body=${core_body#<?php} core_body=${core_body#[[:space:]]} ;;
'<?='*) core_body="echo ${core_body#<?=}" ;;
'<?'*) core_body=${core_body#<?} ;;
esac
emit_interpreter php -r
