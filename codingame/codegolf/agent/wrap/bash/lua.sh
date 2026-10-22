#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_quoted "lua -e" "$core_body"
emit_heredoc lua "$core_body"
