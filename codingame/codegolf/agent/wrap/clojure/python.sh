#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/clojure.sh"
emit clj "$(clojure_wrapper python3 -c "$core_body")"
