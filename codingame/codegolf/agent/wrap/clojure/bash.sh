#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit clj "$(clojure_wrapper sh -c "$core_body")"
emit clj "$(clojure_wrapper bash -c "$core_body")"
