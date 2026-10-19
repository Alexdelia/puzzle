#!/usr/bin/env bash
referee_dir=$(cd "$(dirname "$0")" && pwd)
exec "$referee_dir/../../../agent/tool/referee.sh" "$referee_dir" "$@"
