#!/usr/bin/env python3

import sys


def solve(data: str) -> tuple[int, int]:
	lines = data.splitlines()

	p1_sum = 0

	for line in lines:
		b = [int(x) for x in line.strip()]
		b_len = len(b)

		m: tuple[int, int] = (0, 0)

		for i in range(b_len):
			if i < b_len - 1 and b[i] > m[0]:
				m = (b[i], b[i + 1])
				continue

			if b[i] > m[1]:
				m = (m[0], b[i])

		m: int = m[0] * 10 + m[1]
		print(f"line: {line} -> max: {m}")
		p1_sum += m

	return (p1_sum, 0)


test = """\
987654321111111
811111111111119
234234234234278
818181911112111
"""

expected = (357, 0)
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
