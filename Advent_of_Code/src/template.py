#!/usr/bin/env python3

from aocd import get_data

DAY = 1
YEAR = 2022
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

print(lines)
