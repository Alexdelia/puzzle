#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_perl_exec exec "$core_body"
emit_perl_exec "exec bash,-c=>" "$core_body"
