#!/usr/bin/env bash
set -euo pipefail
venv=${NOVA_VENV:?set NOVA_VENV to a venv with numpy and torch}
export LD_LIBRARY_PATH=/run/opengl-driver/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
exec "$venv/bin/python" "$@"
