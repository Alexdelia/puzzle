#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_go /usr/bin/python3 -c "$core_body"
emit_go /usr/bin/python3 "-c$core_body"
