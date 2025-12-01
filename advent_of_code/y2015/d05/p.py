#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

t = 0

for w in lines:
	if (
		not re.search(r"ab|cd|pq|xy", w)
		and re.search(r"[aeiou].*[aeiou].*[aeiou]", w)
		and re.search(r"(.)\1", w)
	):
		t += 1

print(f"part 1:\t{t}")

t = 0

for w in lines:
	if re.search(r"(..).*\1", w) and re.search(r"(.).\1", w):
		t += 1

print(f"part 2:\t{t}")
