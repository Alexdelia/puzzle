#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
mapfile -t arguments < <(perl_args)
emit_quoted "perl ${arguments[*]}" "$core_body"
file_flags=${arguments[*]}
file_flags=${file_flags%[eE]}
file_flags=${file_flags%-}
file_flags=${file_flags% }
variants=("$file_flags")
if [[ ${arguments[-1]} == *E ]]; then
	variants=()
	for version in 5.01 5.034; do
		if [[ $file_flags == *[0-9alnpsw] && ${file_flags##* } == -* ]]; then
			variants+=("${file_flags}M$version")
		else
			variants+=("${file_flags:+$file_flags }-M$version")
		fi
	done
fi
for flags in "${variants[@]}"; do
	emit_heredoc "perl${flags:+ $flags}" "$core_body"
	emit sh "exec perl${flags:+ $flags} -x \$0"$'\n#!perl\n'"$core_body"
done
