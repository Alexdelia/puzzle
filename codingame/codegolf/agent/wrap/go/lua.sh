#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_go /bin/lua "-e$core_body"
emit_go /bin/sh -c 'lua -e"$0"' "$core_body"
