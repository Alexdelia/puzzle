#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../lib.sh"
source "$(dirname "$0")/common.sh"
emit ts "$core_body"
emit ts "//@ts-nocheck
$core_body"
[[ $core_body != *$'\n'* ]] && emit ts "//@ts-ignore
$core_body"
emit ts "eval($(ts_literal "$core_body"))"
emit ts "eval?.($(ts_literal "$core_body"))"
