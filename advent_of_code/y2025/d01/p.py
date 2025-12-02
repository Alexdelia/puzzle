#!/usr/bin/env python3

import sys
import re

sys.path.append("../..")
from get_data import get_data

DATA: str = get_data()

lines = DATA.splitlines()

dial = 50
zero_count = 0

for line in lines:
	match = re.match(r"(L|R)(\d+)", line)
	if not match:
		raise ValueError(f"Invalid line: {line}")
	dir, value = match.groups()
	n = int(value) * (1 if dir == "R" else -1)

	# print(f"{dial} & '{line}' ->\t", end="")
	dial = (dial + n) % 100
	# print(f"{dial}")

	if dial == 0:
		zero_count += 1

print(zero_count)
