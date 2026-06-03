#!/usr/bin/env python3

from pathlib import Path

GREEN = "\033[0;32m"
RED = "\033[0;31m"
RESET = "\033[0m"

sum_bak = 0.0
sum_cur = 0.0

for d in sorted(
	Path("output.bak").iterdir(),
	key=lambda p: (
		not p.name.isdigit(),
		int(p.name) if p.name.isdigit() else 0,
		p.name,
	),
):
	if not d.is_dir():
		continue
	name = d.name
	ttf_bak = d / "turn_to_finish.txt"
	ttf_cur = Path("output") / name / "turn_to_finish.txt"

	if not ttf_bak.exists() or not ttf_cur.exists():
		continue

	val_bak = float(ttf_bak.read_text().strip())
	val_cur = float(ttf_cur.read_text().strip())

	sum_bak += val_bak
	sum_cur += val_cur

	diff = val_cur - val_bak
	color = GREEN if val_cur <= val_bak else RED
	print(f"{color}{name:>2} {val_bak:7.3f} > {val_cur:7.3f}  {diff:+8.3f}{RESET}")

diff_total = sum_cur - sum_bak
print(f"{sum_bak:.3f} > {sum_cur:.3f}  {diff_total:+.3f}")
