#!/usr/bin/env bash
if [[ -n ${FAKE_CLAUDE_EXEC:-} ]]; then
	eval "$FAKE_CLAUDE_EXEC"
fi
if [[ -n ${FAKE_CLAUDE_SLEEP:-} ]]; then
	sleep "$FAKE_CLAUDE_SLEEP"
fi
cat "$FAKE_CLAUDE_SCRIPT"
