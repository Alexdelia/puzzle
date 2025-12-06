#!/usr/bin/env python3

import sys
import math
from itertools import groupby


def row_num(row: str) -> list[int]:
	return [int(n) for n in row.split()]


def solve(data: str) -> tuple[int, int]:
	lines = list(reversed(data.splitlines()))

	sign_row = lines[0].split()

	number_lines = lines[1:]

	p1_row = row_num(number_lines[0])

	for line in number_lines[1:]:
		for i, n in enumerate(row_num(line)):
			if sign_row[i] == "+":
				p1_row[i] += n
			elif sign_row[i] == "*":
				p1_row[i] *= n

	p1 = sum(p1_row)

	# === part 2 ===

	number_lines = list(reversed(number_lines))

	numbers = [0] * max(len(line) for line in number_lines)

	for line in number_lines:
		for i, c in enumerate(line):
			if c == " ":
				continue

			numbers[i] = numbers[i] * 10 + int(c)

	number_cols = [list(v) for k, v in groupby(numbers, key=lambda x: x != 0) if k != 0]

	p2 = 0

	for numbers, sign in zip(number_cols, sign_row, strict=True):
		if sign == "+":
			p2 += sum(numbers)
		elif sign == "*":
			p2 += math.prod(numbers)

	return (p1, p2)


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
