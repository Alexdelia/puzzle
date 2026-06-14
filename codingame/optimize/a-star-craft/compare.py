#!/usr/bin/env python3

from pathlib import Path

SCORE_FILE = "score.txt"
OUTPUT_DIR = "output"
OUTPUT_COMPARE_DIR = "output.bak"

GREEN = "\033[0;32m"
RED = "\033[0;31m"
RESET = "\033[0m"

sum_cmp = 0
sum_cur = 0

for d in sorted(
	Path(OUTPUT_COMPARE_DIR).iterdir(),
	key=lambda p: (
		not p.name.isdigit(),
		int(p.name) if p.name.isdigit() else 0,
		p.name,
	),
):
	if not d.is_dir():
		continue
	name = d.name
	score_cmp = d / SCORE_FILE
	score_cur = Path(OUTPUT_DIR) / name / SCORE_FILE

	if not score_cmp.exists() or not score_cur.exists():
		continue

	val_cmp = int(score_cmp.read_text().strip())
	val_cur = int(score_cur.read_text().strip())

	sum_cmp += val_cmp
	sum_cur += val_cur

	diff = val_cur - val_cmp
	color = GREEN if val_cur >= val_cmp else RED
	print(f"{color}{name:<25} {val_cmp:4} > {val_cur:4}  {diff:+4}{RESET}")

diff_total = sum_cur - sum_cmp
print(f"{sum_cmp:} > {sum_cur:}  {diff_total:+}")
