#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_clojure sh -c "$core_body"
emit_clojure bash -c "$core_body"
