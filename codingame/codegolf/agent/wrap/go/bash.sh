#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_go /bin/sh -c "$core_body"
emit_go /bin/bash -c "$core_body"
