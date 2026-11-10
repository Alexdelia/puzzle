#!/usr/bin/env bash
trap 'echo saved > "$CONTEST_JOB_DIR/checkpoint"; exit 0' TERM
echo ready
while true; do
	sleep 0.1
done
