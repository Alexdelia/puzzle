#!/usr/bin/env python3

import sys
from pathlib import Path

script_dir = Path(__file__).resolve().parent

validator_dir = script_dir / "validator"
output_dir = script_dir / "output"

SOLUTION_FILE_NAME = "solution.txt"
VALIDATOR_FILE_EXTENSION = ".txt"

SCORE_FILE_NAME = "score.txt"


def read_validator(path: Path) -> tuple[str, str]:
	with path.open() as f:
		op = f.readline().strip()
		rest = f.read()
	packs = [p.strip() for p in rest.strip().split("\n\n")]
	if not packs:
		raise ValueError(f"{path}: expected at least one pack after op line")
	test = packs[0].splitlines()
	if len(test) < 3:
		raise ValueError(f"{path}: test pack must have 3 lines")
	return op, "|".join(test[:3])


def read_solution(path: Path) -> str:
	moves: list[str] = []
	with path.open() as f:
		for raw in f:
			s = raw.strip()
			if not s:
				continue
			a, b = s.split()
			moves.append(f"{int(a)} {int(b)}")
	return ";".join(moves)


parts: list[str] = []
for entry in sorted(output_dir.iterdir()):
	sol_path = entry / SOLUTION_FILE_NAME
	val_path = validator_dir / (entry.name + VALIDATOR_FILE_EXTENSION)
	if not sol_path.is_file():
		continue
	if not val_path.is_file():
		print(f"missing validator for {entry.name}", file=sys.stderr)
		continue

	op, flag = read_validator(val_path)
	sol = read_solution(sol_path)

	parts.append('("' + flag + '","' + op + '","' + sol + '"),')

total = 0.0
for ttf_path in output_dir.glob("*/" + SCORE_FILE_NAME):
	total += float(ttf_path.read_text().strip())

print(f"\033[1;32m{total:.3f}\033[0m", file=sys.stderr)
print("".join(parts), end="")
