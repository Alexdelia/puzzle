#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/clojure.sh"
mapfile -t arguments < <(perl_args)
emit clj "$(clojure_wrapper perl "${arguments[@]}" "$core_body")"
