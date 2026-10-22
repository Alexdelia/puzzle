#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
emit c "main(){system(\"$(escape_c_string "$core_body")\");}"
emit c "main(){system(\"$(escape_c_string "$(shell_command bash -c)")\");}"
