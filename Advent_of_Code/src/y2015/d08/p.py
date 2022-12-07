#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

t1 = 0
t2 = 0

for l in lines:
    s = re.sub(r'\\\\', ' ', l)
    s = re.sub(r'\\"', ' ', s)
    s = re.sub(r'\\x[0-9a-f]{2}', ' ', s)
    t1 += len(l) - len(s[1:-1])

    t2 += 2 + l.count('"') + l.count('\\')

print(f"part 1:\t{t1}")
print(f"part 2:\t{t2}")
