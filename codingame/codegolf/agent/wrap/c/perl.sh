#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
mapfile -t arguments < <(perl_args)
emit c "main(){system(\"$(escape_c_string "$(shell_command perl "${arguments[@]}")")\");}"
