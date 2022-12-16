#!/usr/bin/env python3

import re
from os.path import dirname
from typing import List

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

grid: List[List[int]] = [[int(tree) for tree in l] for l in lines]
visible: List[List[bool]] = [[False for _ in l] for l in lines]

SIZE_X = len(grid)
SIZE_Y = len(grid[0])

for x in range(SIZE_X):
    tallest = -1
    for y in range(SIZE_Y):
        if grid[x][y] > tallest:
            tallest = grid[x][y]
            visible[x][y] = True

for x in range(SIZE_X):
    tallest = -1
    for y in range(SIZE_Y - 1, -1, -1):
        if grid[x][y] > tallest:
            tallest = grid[x][y]
            visible[x][y] = True

for y in range(SIZE_Y):
    tallest = -1
    for x in range(SIZE_X):
        if grid[x][y] > tallest:
            tallest = grid[x][y]
            visible[x][y] = True

for y in range(SIZE_Y):
    tallest = -1
    for x in range(SIZE_X - 1, -1, -1):
        if grid[x][y] > tallest:
            tallest = grid[x][y]
            visible[x][y] = True

for x in range(SIZE_X):
    for y in range(SIZE_Y):
        if visible[x][y]:
            print("\033[32;1m", end='')
        print(grid[x][y], end='\033[0m')
    print()

print(sum(sum(v) for v in visible))
