#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit js "$(exec_sync "$core_body")"
emit_interpreter bash -c
