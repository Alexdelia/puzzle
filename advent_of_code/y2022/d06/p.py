#!/usr/bin/env python3

import re
from os.path import dirname

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

l = 4

for i in range(l - 1, len(DATA)):
	s = set(DATA[i - l : i])
	if len(s) == l:
		if l == 4:
			print(f"part 1:\t{i}")
			l = 14
		else:
			print(f"part 2:\t{i}")
			break
