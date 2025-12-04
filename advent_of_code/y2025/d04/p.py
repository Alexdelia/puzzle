#!/usr/bin/env python3

import sys
from collections.abc import Iterator
from typing import NamedTuple
from time import sleep

NEIGHBORS_THRESHOLD = 4

VISUALIZE_DELAY_SECOND = 0.025


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


def print_grid(grid: list[list[int]]) -> None:
	h = len(grid)
	w = len(grid[0])

	color = [
		"\033[0;2m",
		"\033[0;1;38;2;255;0;0m",
		"\033[0;1;38;2;255;128;0m",
		"\033[0;1;38;2;255;255;0m",
		"\033[0;38;2;0;255;0m",
		"\033[0;38;2;0;255;128m",
		"\033[0;38;2;0;255;255m",
		"\033[0;38;2;0;128;255m",
		"\033[0;38;2;0;0;255m",
	]
	tile = "." + "X" * 3 + "@" * 5
	extra_tile = ["  "] + [" "] * 3 + ["󱅗 "] * 5
	padding = "  "

	# buf = "\033c"
	buf = f"\033[{h + 2}F"

	buf += (
		padding
		+ color[0]
		+ "╭"
		+ "─" * w
		+ "╮"
		+ padding
		+ color[0]
		+ "╭"
		+ "─" * w * 2
		+ "╮\n"
	)

	for row in grid:
		buf += (
			padding
			+ color[0]
			+ "│"
			+ "".join([color[cell] + tile[cell] for cell in row])
			+ color[0]
			+ "│"
			+ padding
			+ color[0]
			+ "│"
			+ "".join([color[cell] + extra_tile[cell] for cell in row])
			+ color[0]
			+ "│\n"
		)
	buf += (
		padding
		+ color[0]
		+ "╰"
		+ "─" * w
		+ "╯"
		+ padding
		+ color[0]
		+ "╰"
		+ "─" * w * 2
		+ "╯\n"
	)

	print(buf, end="")


def remove_paper_rolls(h: int, w: int, lines: list[str], grid: list[list[int]]) -> int:
	count = 0
	new_lines = [list(line) for line in lines]

	for y in range(h):
		for x in range(w):
			if lines[y][x] != "@":
				grid[y][x] = 0
				continue

			grid[y][x] = sum(
				1 for n in neighbors(w, h, Coord(x, y)) if lines[n.y][n.x] == "@"
			)

			if grid[y][x] < NEIGHBORS_THRESHOLD:
				new_lines[y][x] = "."
				count += 1

	lines[:] = ["".join(line) for line in new_lines]
	return count


def solve(data: str, visualize: bool = False) -> tuple[int, int]:
	lines = data.splitlines()
	h = len(lines)
	w = len(lines[0])

	grid = [[0 for _ in range(w)] for _ in range(h)]

	p1 = remove_paper_rolls(h, w, lines, grid)
	if visualize:
		print("\033c", end="")
		print_grid(grid)
		sleep(VISUALIZE_DELAY_SECOND)

	p2 = p1
	while removed := remove_paper_rolls(h, w, lines, grid):
		p2 += removed
		if visualize:
			print_grid(grid)
			sleep(VISUALIZE_DELAY_SECOND)

	if visualize:
		print_grid(grid)

	return (p1, p2)


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
expected = (13, 43)
got = solve(test)
assert expected[0] == got[0], (
	f"part 1 test failed: expected {expected[0]}, got {got[0]}"
)
assert expected[1] == got[1], (
	f"part 2 test failed: expected {expected[1]}, got {got[1]}"
)

sys.path.append("../..")
from get_data import get_data

p1, p2 = solve(get_data(), visualize=True)
print(f"part 1:\t{p1}")
print(f"part 2:\t{p2}")
