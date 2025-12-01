#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

t1 = 0
t2 = 0

for l in lines:
	p = l.split(",")
	r1 = p[0].split("-")
	r2 = p[1].split("-")
	r1 = set(range(int(r1[0]), int(r1[1]) + 1))
	r2 = set(range(int(r2[0]), int(r2[1]) + 1))

	if r1.issubset(r2) or r2.issubset(r1):
		t1 += 1

	if r1.intersection(r2):
		t2 += 1

print(f"part 1:\t{t1}")
print(f"part 2:\t{t2}")
