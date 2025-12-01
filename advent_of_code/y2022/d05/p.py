#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

sep = 0
while sep < len(lines) and lines[sep] != "":
	sep += 1

stacks = lines[: sep - 1]
lines = lines[sep + 1 :]

s1 = [[] for _ in range(9)]

for i in range(len(stacks) - 1, -1, -1):
	for c in range(1, len(stacks[i]), 4):
		if stacks[i][c] != " ":
			s1[(c - 1) // 4].append(stacks[i][c])

# make a non deep copy of s1 to s2
s2 = [s1[i][:] for i in range(len(s1))]

for l in lines:
	n, src, dst = re.sub(r"[^0-9]", " ", l).split()
	n = int(n)
	src = int(src) - 1
	dst = int(dst) - 1

	for _ in range(n):
		s1[dst].append(s1[src].pop())

	tmp = []
	for _ in range(n):
		tmp.append(s2[src].pop())
	for _ in range(n):
		s2[dst].append(tmp.pop())

print(f"part 1:\t{''.join([i[-1] for i in s1])}")
print(f"part 2:\t{''.join([i[-1] for i in s2])}")
