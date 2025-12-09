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


def vertical_segment_cross_rectangle(
	ra: Coord, rb: Coord, sx: int, sy1: int, sy2: int
) -> bool:
	if sx in (ra.x, rb.x):
		return False

	if sx < min(ra.x, rb.x) or sx > max(ra.x, rb.x):
		return False

	return not (max(sy1, sy2) < min(ra.y, rb.y) or min(sy1, sy2) > max(ra.y, rb.y))


def horizontal_segment_cross_rectangle(
	ra: Coord, rb: Coord, sy: int, sx1: int, sx2: int
) -> bool:
	if sy in (ra.y, rb.y):
		return False

	if sy < min(ra.y, rb.y) or sy > max(ra.y, rb.y):
		return False

	return not (max(sx1, sx2) < min(ra.x, rb.x) or min(sx1, sx2) > max(ra.x, rb.x))


def segment_cross_rectangle(ra: Coord, rb: Coord, sa: Coord, sb: Coord) -> bool:
	if sa.x == sb.x:
		return vertical_segment_cross_rectangle(ra, rb, sa.x, sa.y, sb.y)

	return horizontal_segment_cross_rectangle(ra, rb, sa.y, sa.x, sb.x)


def rectangle_cross_any_segment(
	ra: Coord, rb: Coord, segments: list[tuple[Coord, Coord]]
) -> bool:
	return any(segment_cross_rectangle(ra, rb, sa, sb) for sa, sb in segments)


def solve(data: str) -> tuple[int, int]:
	lines = data.splitlines()

	coord_list = [Coord.parse(line) for line in lines]

	segment_list = [
		(coord_list[i], coord_list[i + 1]) for i in range(len(coord_list) - 1)
	]
	segment_list.append((coord_list[-1], coord_list[0]))

	p1 = 0
	p2 = 0

	for x in range(len(coord_list)):
		for y in range(x + 1, len(coord_list)):
			a = area(coord_list[x], coord_list[y])

			p1 = max(p1, a)

			if p2 < a and not rectangle_cross_any_segment(
				coord_list[x],
				coord_list[y],
				segment_list,
			):
				p2 = a

	return (p1, p2)


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
expected = (50, 24)
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
