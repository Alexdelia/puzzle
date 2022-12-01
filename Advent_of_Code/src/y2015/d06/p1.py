#!/usr/bin/env python3

import re
from os.path import dirname
from typing import List

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

grid: List[List[bool]] = [[False for _ in range(1000)] for _ in range(1000)]

for l in lines:
    action, start, _, end = re.sub("turn ", "", l).split()

    start = [int(x) for x in start.split(",")]
    end = [int(x) for x in end.split(",")]

    for x in range(start[0], end[0] + 1):
        for y in range(start[1], end[1] + 1):
            if action == "on":
                grid[x][y] = True
            elif action == "off":
                grid[x][y] = False
            else:
                grid[x][y] = not grid[x][y]

t = 0

for x in range(1000):
    for y in range(1000):
        if grid[x][y]:
            t += 1

print(t)
