#!/usr/bin/env python3

import base64
import sys
from pathlib import Path

script_dir = Path(__file__).resolve().parent

validator_dir = script_dir / "validator"
output_dir = script_dir / "output"

SOLUTION_FILE_NAME = "solution.txt"
VALIDATOR_FILE_EXTENSION = ".txt"

TILT_OFFSET = 18

parts = []
for entry in sorted(output_dir.iterdir()):
	sol_path = entry / SOLUTION_FILE_NAME
	flag_path = validator_dir / (entry.name + VALIDATOR_FILE_EXTENSION)
	if not sol_path.is_file():
		continue
	with flag_path.open() as f:
		flag = f.readline().rstrip("\n")
	buf = bytearray()
	with sol_path.open() as f:
		for raw_line in f:
			stripped = raw_line.strip()
			if not stripped:
				continue
			a, b = stripped.split()
			buf.append(int(a) + TILT_OFFSET)
			buf.append(int(b))
	compressed = base64.b64encode(buf).decode()
	parts.append('("' + flag + '","' + compressed + '"),')

total = 0.0
for ttf_path in output_dir.glob("*/turn_to_finish.txt"):
	total += float(ttf_path.read_text().strip())

print(f"\033[1;32m{total:.3f}\033[0m", file=sys.stderr)
print("".join(parts), end="")
