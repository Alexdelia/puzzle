#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_ocaml_command "$core_body"
emit_ocaml_command "$(shell_command bash -c)"
