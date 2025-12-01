#!/usr/bin/env python3

import re
from os.path import dirname
from typing import List, Set, Tuple

from aocd import get_data

Coord = Tuple[int, int]

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

sensors: Set[Tuple[Coord, Coord]] = set()

for l in lines:
	l = re.sub(r"[^0-9,:-]", "", l)
	s, b = l.split(":")
	sx, sy = map(int, s.split(","))
	bx, by = map(int, b.split(","))

	sensors.add(((sx, sy), (bx, by)))

assert len(sensors) == len(lines)


def dist(a: Coord, b: Coord) -> int:
	return abs(b[0] - a[0]) + abs(b[1] - a[1])


def merge_range(r: List[Tuple[int, int]]) -> List[Tuple[int, int]]:
	r.sort(key=lambda x: x[0])
	ret = []

	for i in range(len(r) - 1):
		if r[i + 1][0] <= r[i][1] <= r[i + 1][1] or r[i + 1][0] == r[i][1] + 1:
			r[i + 1] = (r[i][0], r[i + 1][1])
		elif r[i][0] <= r[i + 1][0] and r[i][1] >= r[i + 1][1]:
			r[i + 1] = r[i]
			continue
		else:
			ret.append(r[i])

	ret.append(r[-1])
	return ret


def process_row(row: int) -> List[Tuple[int, int]]:
	r: List[Tuple[int, int]] = []

	for s in sensors:
		n = dist(s[0], s[1]) - abs(row - s[0][1])

		if n < 0:
			continue

		r.append((s[0][0] - n, s[0][0] + n))

	return merge_range(r)


r = process_row(2_000_000)[0]
print(f"part 1:\t{r[1] - r[0]}")

for row in range(4_000_001):
	print(f"{row}/{4_000_000}", end="\r")
	r = process_row(row)
	if len(r) > 1:
		print(f"\npart 2:\t{4_000_000 * (r[0][1] + 1) + row}")
		break
