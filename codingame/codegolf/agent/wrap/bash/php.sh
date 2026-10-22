#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
file_text=$core_text
[[ $file_text == '<?'[^p=]* ]] && file_text="<?php ${file_text#<?}"
for php in php /bin/php; do
	for options in "" " -ddisplay_errors="; do
		emit_heredoc "$php$options" $'\n'"$file_text"
	done
done
[[ $core_body == '<?'* && $core_body != *'?>'* ]] || exit 0
case $core_body in
'<?php'[[:space:]]*) core_body=${core_body#<?php} ;;
'<?='*) core_body="echo ${core_body#<?=}" ;;
*) core_body=${core_body#<?} ;;
esac
for php in php /bin/php; do
	for options in "" " -ddisplay_errors="; do
		emit_quoted "$php$options -r" "$core_body"
	done
done
