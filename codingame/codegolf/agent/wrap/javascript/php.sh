#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
php_code=$core_text
if [[ $php_code =~ ^\<\?php[[:space:]] ]]; then
	php_code=${php_code:6}
elif [[ $php_code == '<?='* ]]; then
	php_code="echo ${php_code:3}"
elif [[ $php_code == '<?'* ]]; then
	php_code=${php_code:2}
fi
core_body=$php_code
emit_interpreter php -r
