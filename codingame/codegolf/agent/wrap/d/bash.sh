#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_d_call spawnShell "$(d_string "$core_body")"
emit_d bash -c
