#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_quoted "node -e" "$core_body"
emit_heredoc node "$core_body"
