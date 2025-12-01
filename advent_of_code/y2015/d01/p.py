#!/usr/bin/env python3

from aocd import get_data

DAY = 1
YEAR = 2015
DATA: str = get_data(day=DAY, year=YEAR)

r = 0
bi = 0

for i, c in enumerate(DATA):
	if c == "(":
		r += 1
	elif c == ")":
		r -= 1
	if r == -1 and bi == 0:
		bi = i + 1

print(f"part 1:\t{r}")
print(f"part 2:\t{bi}")
