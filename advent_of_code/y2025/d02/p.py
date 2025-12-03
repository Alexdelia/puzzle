#!/usr/bin/env python3

import re
import sys

sys.path.append("../..")
from get_data import get_data

DATA: str = get_data()

ranges: list[str] = DATA.split(",")


def solve(ranges: list[str]) -> tuple[int, int]:
	regex = re.compile(r"^(\d+)\1$")

	match_sum = 0

	for r in ranges:
		start, end = map(int, r.split("-"))

		for n in range(start, end + 1):
			if regex.search(str(n)):
				match_sum += n

	return (match_sum, 0)


test_ranges = [
	"11-22",
	"95-115",
	"998-1012",
	"1188511880-1188511890",
	"222220-222224",
	"1698522-1698528",
	"446443-446449",
	"38593856-38593862",
	"565653-565659",
	"824824821-824824827",
	"2121212118-2121212124",
]
expected = (1227775554, 0)
got = solve(test_ranges)
assert expected[0] == got[0], (
	f"part 1 test failed: expected {expected[0]}, got {got[0]}"
)
assert expected[1] == got[1], (
	f"part 2 test failed: expected {expected[1]}, got {got[1]}"
)

p1, p2 = solve(ranges)
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
