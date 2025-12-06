#!/usr/bin/env python3

import math
import sys


def row_num(row: str) -> list[int]:
	return [int(n) for n in row.split()]


def solve(data: str) -> tuple[int, int]:
	lines = list(reversed(data.splitlines()))

	sign_row = lines[0].split()
	numbers = lines[1:]

	p1_row = row_num(numbers[0])
	p2_col = []

	for x, line in enumerate(numbers):
		number_row = row_num(line)

		if x > 0:
			for y, n in enumerate(number_row):
				if sign_row[y] == "+":
					p1_row[y] += n
				elif sign_row[y] == "*":
					p1_row[y] *= n

		if sign_row[x] == "+":
			p2_col.append(sum(number_row))
		elif sign_row[x] == "*":
			p2_col.append(math.prod(number_row))

	return (sum(p1_row), sum(p2_col))


test = """\
123 328  51 64
 45 64  387 23
  6 98  215 314
*   +   *   +
"""
expected = (4277556, 3263827)
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
