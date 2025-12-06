#!/usr/bin/env python3

import sys


def solve(data: str) -> tuple[int, int]:
	lines = list(reversed(data.splitlines()))

	sign_row = lines[0].split()

	res_row = [int(n) for n in lines[1].split()]

	for line in lines[2:]:
		number_row = [int(n) for n in line.split()]
		for i, n in enumerate(number_row):
			if sign_row[i] == "+":
				res_row[i] += n
			elif sign_row[i] == "*":
				res_row[i] *= n

	return (sum(res_row), 0)


test = """\
123 328  51 64 
 45 64  387 23 
  6 98  215 314
*   +   *   +
"""
expected = (4277556, 0)
got = solve(test)
assert expected[0] == got[0], (
	f"part 1 test failed: expected {expected[0]}, got {got[0]}"
)
assert expected[1] == got[1], (
	f"part 2 test failed: expected {expected[1]}, got {got[1]}"
)

sys.path.append("../..")
from get_data import get_data

DATA: str = get_data()

p1, p2 = solve(DATA)
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
