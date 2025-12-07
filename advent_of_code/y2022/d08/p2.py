#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

grid: list[list[int]] = [[int(tree) for tree in l] for l in lines]
scenic = -1

SIZE_X = len(grid)
SIZE_Y = len(grid[0])

for x in range(SIZE_X):
	for y in range(SIZE_Y):
		# count number of trees smaller than this one in the 4 directions
		total = 1
		for dx, dy in [(1, 0), (0, 1), (-1, 0), (0, -1)]:
			score = 0
			nx, ny = x + dx, y + dy
			while 0 <= nx < SIZE_X and 0 <= ny < SIZE_Y:
				score += 1
				if grid[x][y] <= grid[nx][ny]:
					break
				nx, ny = nx + dx, ny + dy
			total *= score
		if total > scenic:
			scenic = total

print(scenic)
