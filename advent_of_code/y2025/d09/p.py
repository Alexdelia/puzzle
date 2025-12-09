#!/usr/bin/env python3

import sys
from typing import NamedTuple


class Coord(NamedTuple):
	x: int
	y: int

	@staticmethod
	def parse(s: str) -> "Coord":
		x, y = s.split(",")
		return Coord(int(x), int(y))


def area(a: Coord, b: Coord) -> int:
	return (abs(a.x - b.x) + 1) * (abs(a.y - b.y) + 1)


def solve(data: str) -> tuple[int, int]:
	lines = data.splitlines()

	coord_list = [Coord.parse(line) for line in lines]

	max_area = 0
	for x in range(len(coord_list)):
		for y in range(x + 1, len(coord_list)):
			max_area = max(max_area, area(coord_list[x], coord_list[y]))

	return (max_area, 0)


expected_area = 50
got_area = area(Coord(2, 5), Coord(11, 1))
assert expected_area == got_area, (
	f"area test failed: expected {expected_area}, got {got_area}"
)
test = """\
7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3
"""
expected = (50, 0)
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
