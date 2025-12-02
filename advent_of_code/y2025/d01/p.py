#!/usr/bin/env python3

import re
import sys

sys.path.append("../..")
from get_data import get_data

DATA: str = get_data()

lines = DATA.splitlines()

DIAL_SIZE = 100


def solve(lines: list[str]) -> tuple[int, int]:
	dial = 50
	zero_count = 0
	click_count = 0

	for line in lines:
		assert line[1] != "0", f"move '{line}' is undefined"

		match = re.match(r"(L|R)(\d+)", line)
		if not match:
			raise ValueError(f"Invalid line: {line}")
		rot, value = match.groups()
		n = int(value) * (1 if rot == "R" else -1)

		to = dial + n

		if to == 0:
			click_count += 1
		elif to >= DIAL_SIZE:
			click_count += to // DIAL_SIZE
		elif to < 0:
			click_count += abs((to - 1) // DIAL_SIZE)
			if dial == 0:
				click_count -= 1

		print(f"'{line}' ({n}):\t{dial} -> {to}\t(clicks: {click_count})")
		dial = to % DIAL_SIZE

		if dial == 0:
			zero_count += 1

	return (zero_count, click_count)


test_lines = [
	"L68",
	"L30",
	"R48",
	"L5",
	"R60",
	"L55",
	"L1",
	"L99",
	"R14",
	"L82",
]
expected = (3, 6)
got = solve(test_lines)
assert expected[0] == got[0], (
	f"part 1 test failed: expected {expected[0]}, got {got[0]}"
)
assert expected[1] == got[1], (
	f"part 2 test failed: expected {expected[1]}, got {got[1]}"
)


p1, p2 = solve(lines)
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
