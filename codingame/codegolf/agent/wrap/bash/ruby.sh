#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
mapfile -t arguments < <(perl_args)
for ruby in /usr/local/bin/ruby; do
	emit_quoted "$ruby ${arguments[*]}" "$core_body"
	file_flags=${arguments[*]}
	file_flags=${file_flags%e}
	file_flags=${file_flags%-}
	file_flags=${file_flags% }
	emit_heredoc "$ruby${file_flags:+ $file_flags}" "$core_body"
	emit sh "exec $ruby${file_flags:+ $file_flags} -x \$0"$'\n#!ruby\n'"$core_body"
done
