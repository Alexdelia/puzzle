#!/usr/bin/env python3

import re
from os.path import dirname
from typing import Dict, List, Set, Tuple

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

d: Dict[Tuple[str, str], int] = {}

for l in lines:
	src, _, dst, _, dist = l.split()
	d[tuple(sorted([src, dst]))] = int(dist)

s: Set[str] = set()
for k in d.keys():
	s.add(k[0])
	s.add(k[1])

n = len(s)


# create all sequences of cities
def seqs(s: Set[str], n: int) -> List[List[str]]:
	if n == 1:
		return [[x] for x in s]
	else:
		res = []
		for x in s:
			s2 = s.copy()
			s2.remove(x)
			for y in seqs(s2, n - 1):
				res.append([x] + y)
		return res


# calculate distance of a sequence
def seq_dist(s: List[str]) -> int:
	res = 0
	for i in range(len(s) - 1):
		res += d[tuple(sorted([s[i], s[i + 1]]))]
	return res


dists = [seq_dist(x) for x in seqs(s, n)]

print(f"part 1:\t{min(dists)}")
print(f"part 2:\t{max(dists)}")
