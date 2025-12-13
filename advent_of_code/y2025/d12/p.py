#!/usr/bin/env python3

import sys

STARTING_INDEX = 30


def solve(data: str) -> tuple[int, int]:
	lines = data.splitlines()

	p1 = 0

	for line in lines[STARTING_INDEX:]:
		d, *b = line.split()

		w, h = map(int, d[:-1].split("x"))
		b = sum(map(int, b))

		if w * h >= b * 9:
			p1 += 1

	return (p1, 0)


sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data())
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
