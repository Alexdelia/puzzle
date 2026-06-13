#!/usr/bin/env python3

import sys
from pathlib import Path

script_dir = Path(__file__).resolve().parent

validator_dir = script_dir / "validator"
output_dir = script_dir / "output"

SOLUTION_FILE_NAME = "solution.txt"
VALIDATOR_FILE_EXTENSION = ".txt"

SCORE_FILE_NAME = "score.txt"


Pt = tuple[int, int]


def parse_line(s: str) -> list[Pt]:
	s = s.strip()
	if not s:
		return []
	pts: list[Pt] = []
	for token in s.split(";"):
		a, b = token.split()
		pts.append((int(a), int(b)))
	return pts


def read_validator(path: Path) -> tuple[Pt, list[Pt], list[Pt]]:
	with path.open() as f:
		f.readline()
		rest = f.read()
	packs = [p.strip() for p in rest.strip().split("\n\n") if p.strip()]
	if not packs:
		raise ValueError(f"{path}: expected at least one pack after op line")
	val = packs[-1].splitlines()
	if len(val) < 3:
		raise ValueError(f"{path}: validator pack must have 3 lines")
	return parse_line(val[0])[0], parse_line(val[1]), parse_line(val[2])


def read_solution(path: Path) -> list[Pt]:
	moves: list[Pt] = []
	with path.open() as f:
		for raw in f:
			s = raw.strip()
			if not s:
				continue
			a, b = s.split()
			moves.append((int(a), int(b)))
	return moves


def fmt_pt(p: Pt) -> str:
	return f"({p[0]},{p[1]})"


def fmt_pts(pts: list[Pt]) -> str:
	return "vec![" + ",".join(fmt_pt(p) for p in pts) + "]"


parts: list[str] = []
for entry in sorted(output_dir.iterdir()):
	sol_path = entry / SOLUTION_FILE_NAME
	val_path = validator_dir / (entry.name + VALIDATOR_FILE_EXTENSION)
	if not sol_path.is_file():
		continue
	if not val_path.is_file():
		print(f"missing validator for {entry.name}", file=sys.stderr)
		continue

	player, humans, zombies = read_validator(val_path)
	moves = read_solution(sol_path)

	flag = f"({fmt_pt(player)},{fmt_pts(humans)},{fmt_pts(zombies)})"
	parts.append(f"({flag},{fmt_pts(moves)}),")

total = 0.0
for ttf_path in output_dir.glob("*/" + SCORE_FILE_NAME):
	total += float(ttf_path.read_text().strip())

print(f"\033[1;32m{total:.3f}\033[0m", file=sys.stderr)
print("".join(parts), end="")
