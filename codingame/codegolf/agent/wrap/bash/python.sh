#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_quoted "python3 -c" "$core_body"
emit_heredoc python3 "$core_body"
emit sh "exec python3 -x \$0"$'\n'"$core_body"
