#!/usr/bin/env python3

import sys

DEBUG = False

PART_1_SIZE = 2
PART_2_SIZE = 12


def solve_single_battery(line: str) -> tuple[int, int]:
	b = list(map(int, line))
	b_len = len(b)

	def solve_with_size(size: int) -> int:
		m = [0] * size

		for x in range(b_len):
			DEBUG and print(f"b: {line}, m: {m}, x: {x}")

			for y in range(size):
				max_index = b_len - (size - 1 - y)

				DEBUG and print(
					f"- checking b[{x}]={b[x]} > m[{y}]={m[y]} (max_index: {max_index})"
				)

				if x < max_index and b[x] > m[y]:
					to_copy = min(size - y, b_len - x)
					# m[y : y + to_copy] = b[x : x + to_copy]
					m = m[:y] + [b[x]] + ([0] * (to_copy - 1))

					DEBUG and print(f"  - updated m: {m}")

					break

		return int("".join(map(str, m)))

	return (
		solve_with_size(PART_1_SIZE),
		solve_with_size(PART_2_SIZE),
	)


def solve(data: str) -> tuple[int, int]:
	lines = data.splitlines()

	res = [solve_single_battery(line) for line in lines]
	return tuple(sum(n) for n in zip(*res, strict=True))


test = """\
987654321111111
811111111111119
234234234234278
818181911112111
"""
expected_per_line = [
	(98, 987654321111),
	(89, 811111111119),
	(78, 434234234278),
	(92, 888911112111),
]
for i, line in enumerate(test.splitlines()):
	got = solve_single_battery(line)
	expected = expected_per_line[i]
	assert expected == got, (
		f"line[{i}]={line} test failed: expected {expected}, got {got}"
	)

expected = (357, 3121910778619)
got = solve(test)
assert expected[0] == got[0], (
	f"part 1 test failed: expected {expected[0]}, got {got[0]}"
)
assert expected[1] == got[1], (
	f"part 2 test failed: expected {expected[1]}, got {got[1]}"
)

sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data())
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
