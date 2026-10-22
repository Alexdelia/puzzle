#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_rust_system "$core_body"
emit_rust_command sh -c "$core_body"
emit_rust bash -c "$core_body"
