#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit c "$(c_system "$core_body")"
emit_system bash -c
