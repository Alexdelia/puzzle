#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
ruby_exec "" "$core_body"
ruby_exec_interpreter bash -c "$core_body"
