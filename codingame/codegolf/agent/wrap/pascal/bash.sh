#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_system "$core_body"
emit_system "$(shell_command bash -c)"
emit_pascal "fpexeclp('bash',['-c',$(pascal_string "$core_body")])"
