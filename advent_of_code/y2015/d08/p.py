#!/usr/bin/env python3

import re
from pathlib import Path

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", Path(__file__).parent.name))
YEAR = int(re.sub(r"[^0-9]", "", Path(__file__).parent.parent.name))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

s = "".join(lines)

t = len(
	re.sub(
		r'"',
		"",
		re.sub(r"\\x[0-9a-f]{2}", " ", re.sub(r'\\"', " ", re.sub(r"\\\\", " ", s))),
	)
)
t = len(s) - t
print(f"part 1:\t{t}")

t = 2 * len(lines) + s.count('"') + s.count("\\")
print(f"part 2:\t{t}")
