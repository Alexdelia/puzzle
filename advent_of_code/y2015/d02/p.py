#!/usr/bin/env python3

from aocd import get_data

DAY = 2
YEAR = 2015
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

sq = 0
ft = 0

for i in lines:
	l, w, h = i.split("x")
	l, w, h = int(l), int(w), int(h)
	sq += 2 * l * w + 2 * w * h + 2 * h * l + min(l * w, w * h, h * l)
	ft += 2 * min(l + w, w + h, h + l) + l * w * h

print(f"part 1:\t{sq}")
print(f"part 2:\t{ft}")
