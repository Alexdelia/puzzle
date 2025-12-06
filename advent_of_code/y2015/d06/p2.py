#!/usr/bin/env python3

import re
from pathlib import Path

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", Path(__file__).parent.name))
YEAR = int(re.sub(r"[^0-9]", "", Path(__file__).parent.parent.name))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

grid: list[list[int]] = [[0 for _ in range(1000)] for _ in range(1000)]

for l in lines:
	action, start, _, end = re.sub("turn ", "", l).split()

	start = [int(x) for x in start.split(",")]
	end = [int(x) for x in end.split(",")]

	for x in range(start[0], end[0] + 1):
		for y in range(start[1], end[1] + 1):
			if action == "on":
				grid[x][y] += 1
			elif action == "off":
				grid[x][y] = max(0, grid[x][y] - 1)
			else:
				grid[x][y] += 2

t = 0

for x in range(1000):
	for y in range(1000):
		t += grid[x][y]

print(t)
