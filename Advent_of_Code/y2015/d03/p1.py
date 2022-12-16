#!/usr/bin/env python3

from aocd import get_data

DAY = 3
YEAR = 2015
DATA: str = get_data(day=DAY, year=YEAR)

SIZE = 5000

m = [[0 for _ in range(SIZE)] for _ in range(SIZE)]

x = SIZE // 2
y = SIZE // 2

m[x][y] = 1

for i in DATA:
    if i == '^':
        y -= 1
    elif i == 'v':
        y += 1
    elif i == '<':
        x -= 1
    elif i == '>':
        x += 1
    m[x][y] += 1

t = 0

for x in range(SIZE):
    for y in range(SIZE):
        if m[x][y] > 0:
            t += 1

print(f"{t}")
