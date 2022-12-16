#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

t = 0

def priority(s: set) -> int:
    assert len(s) == 1
    x = s.pop()
    if 'a' <= x <= 'z':
        return ord(x) - ord('a') + 1
    else:
        return ord(x) - ord('A') + 27

for l in lines:
    t += priority(set(l[len(l) // 2:]).intersection(set(l[:len(l) // 2])))

print(f"part 1:\t{t}")

t = 0

for i in range(0, len(lines), 3):
    t += priority(set(lines[i]).intersection(set(lines[i + 1])).intersection(set(lines[i + 2])))

print(f"part 2:\t{t}")