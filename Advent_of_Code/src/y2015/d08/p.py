#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

s = ''.join(lines)

t = len(
        re.sub(r'"', '', 
        re.sub(r'\\x[0-9a-f]{2}', ' ',
        re.sub(r'\\"', ' ',
        re.sub(r'\\\\', ' ', 
        s
        ))))
    )
t = len(s) - t
print(f"part 1:\t{t}")

t = 2 * len(lines) + s.count('"') + s.count('\\')
print(f"part 2:\t{t}")
