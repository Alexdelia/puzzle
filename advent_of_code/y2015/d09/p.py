#!/usr/bin/env python3

import re
from pathlib import Path

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", Path(__file__).parent.name))
YEAR = int(re.sub(r"[^0-9]", "", Path(__file__).parent.parent.name))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

d: dict[tuple[str, str], int] = {}

for l in lines:
	src, _, dst, _, dist = l.split()
	d[tuple(sorted([src, dst]))] = int(dist)

s: set[str] = set()
for k in d:
	s.add(k[0])
	s.add(k[1])

n = len(s)


# create all sequences of cities
def seqs(s: set[str], n: int) -> list[list[str]]:
	if n == 1:
		return [[x] for x in s]
	res: list[list[str]] = []
	for x in s:
		s2 = s.copy()
		s2.remove(x)
		for y in seqs(s2, n - 1):
			res.append([x, *y])  # noqa: PERF401
	return res


# calculate distance of a sequence
def seq_dist(s: list[str]) -> int:
	res = 0
	for i in range(len(s) - 1):
		res += d[tuple(sorted([s[i], s[i + 1]]))]
	return res


dists = [seq_dist(x) for x in seqs(s, n)]

print(f"part 1:\t{min(dists)}")
print(f"part 2:\t{max(dists)}")
