#!/usr/bin/env python3

import sys
from pathlib import Path

script_dir = Path(__file__).resolve().parent

validator_dir = script_dir / "validator"
output_dir = script_dir / "output"

SOLUTION_FILE_NAME = "solution.txt"
SCORE_FILE_NAME = "score.txt"
VALIDATOR_FILE_EXTENSION = ".txt"


def rust_literal(text: str) -> str:
	escaped = text.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
	return '"' + escaped + '"'


parts = []
for entry in sorted(output_dir.iterdir()):
	sol_path = entry / SOLUTION_FILE_NAME
	flag_path = validator_dir / (entry.name + VALIDATOR_FILE_EXTENSION)
	if not sol_path.is_file() or not flag_path.is_file():
		continue
	flag = flag_path.read_text().rstrip("\n")
	solution = sol_path.read_text().strip()
	parts.append("(" + rust_literal(flag) + "," + rust_literal(solution) + "),")

total = 0.0
for score_path in output_dir.glob("*/" + SCORE_FILE_NAME):
	total += float(score_path.read_text().strip())

print(f"\033[1;32m{total:.0f}\033[0m", file=sys.stderr)
print("".join(parts), end="")
