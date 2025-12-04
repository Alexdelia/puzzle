#!/usr/bin/env python3

import sys
from collections.abc import Iterator
from typing import NamedTuple

PART_1_NEIGHBORS_THRESHOLD = 4


class Coord(NamedTuple):
	x: int
	y: int


NEIGHBORS = [
	(-1, -1),
	(0, -1),
	(1, -1),
	(-1, 0),
	(1, 0),
	(-1, 1),
	(0, 1),
	(1, 1),
]


def neighbors(w: int, h: int, pos: Coord) -> Iterator[Coord]:
	for dx, dy in NEIGHBORS:
		nx, ny = pos.x + dx, pos.y + dy
		if 0 <= nx < w and 0 <= ny < h:
			yield Coord(nx, ny)


def solve(data: str) -> tuple[int, int]:
	lines = data.splitlines()
	w = len(lines[0])
	h = len(lines)

	grid = [[0 for _ in range(w)] for _ in range(h)]

	p1 = 0

	for y in range(h):
		for x in range(w):
			if lines[y][x] != "@":
				continue

			grid[y][x] = sum(
				1 for n in neighbors(w, h, Coord(x, y)) if lines[n.y][n.x] == "@"
			)

			if grid[y][x] < PART_1_NEIGHBORS_THRESHOLD:
				p1 += 1

	tile = [
		" ",
		"\033[0;1;38;2;255;0;0mX\033[0m",
		"\033[0;1;38;2;255;128;0mX\033[0m",
		"\033[0;1;38;2;255;255;0mX\033[0m",
		"\033[0;38;2;0;255;0m@\033[0m",
		"\033[0;38;2;0;255;128m@\033[0m",
		"\033[0;38;2;0;255;255m@\033[0m",
		"\033[0;38;2;0;128;255m@\033[0m",
		"\033[0;38;2;0;0;255m@\033[0m",
	]
	print("╭" + "─" * w + "╮")
	for y in range(h):
		print("│" + "".join([tile[grid[y][x]] for x in range(w)]) + "│")
	print("╰" + "─" * w + "╯")

	return (p1, 0)


test = """\
..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.
"""
expected = (13, 0)
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
