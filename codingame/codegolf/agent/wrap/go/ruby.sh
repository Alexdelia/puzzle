#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
mapfile -t arguments < <(perl_args)
emit_go /usr/local/bin/ruby "${arguments[@]}" "$core_body"
emit_go /usr/local/bin/ruby "${arguments[@]::${#arguments[@]}-1}" "${arguments[-1]}$core_body"
