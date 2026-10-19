#!/usr/bin/env bash
cd "$(dirname "$0")"
exec agent/tool/submit.sh "$@"
