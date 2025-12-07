#!/usr/bin/env python3

import re
import sys


def solve(data: str) -> tuple[int, int]:
	regex = re.compile(r"(\-?\d+)")

	p1 = 0

	for match in regex.findall(data):
		p1 += int(match)

	return (p1, 0)


sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data())
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
