#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_swift_command lua -e
