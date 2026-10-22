#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit_cpp "$core_body"
dash_body=${core_body//read;/read a;}
[[ $dash_body == "$core_body" ]] || emit_cpp "$dash_body"
emit_cpp_command bash -c
