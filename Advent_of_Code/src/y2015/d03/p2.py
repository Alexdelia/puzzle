#!/usr/bin/env python3

from aocd import get_data

DAY = 3
YEAR = 2015
DATA: str = get_data(day=DAY, year=YEAR)

SIZE = 5000

m = [[0 for _ in range(SIZE)] for _ in range(SIZE)]

x = xr = SIZE // 2
y = yr = SIZE // 2

m[x][y] = 2

for i, c in enumerate(DATA):
    if i % 2 == 0:
        if c == '^':
            y -= 1
        elif c == 'v':
            y += 1
        elif c == '<':
            x -= 1
        elif c == '>':
            x += 1
        m[x][y] += 1
    else:
        if c == '^':
            yr -= 1
        elif c == 'v':
            yr += 1
        elif c == '<':
            xr -= 1
        elif c == '>':
            xr += 1
        m[xr][yr] += 1

t = 0

for x in range(SIZE):
    for y in range(SIZE):
        if m[x][y] > 0:
            t += 1

print(f"{t}")
