#!/usr/bin/env python3

from aocd import get_data

DAY = 1
YEAR = 2022
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

cur = 0
v = []

for l in lines:
    if len(l) > 0:
        cur += int(l)
    else:
        v.append(cur)
        cur = 0

v.sort()
print(v, end="\n\n")

t = 0

print("part 1:", v[-1])

for i in range(3):
    t += v[-i - 1]

print("part 2:", t)

