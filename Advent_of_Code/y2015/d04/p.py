#!/usr/bin/env python3

import re
from hashlib import md5
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-1]))
YEAR = int(re.sub(r'[^0-9]', "", dirname(__file__).split('/')[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

i = 1

while not (md5(f"{DATA}{i}".encode()).hexdigest()[:5] == "00000"):
    i += 1

print(f"part 1:\t{i}")

while not (md5(f"{DATA}{i}".encode()).hexdigest()[:6] == "000000"):
    i += 1

print(f"part 2:\t{i}")