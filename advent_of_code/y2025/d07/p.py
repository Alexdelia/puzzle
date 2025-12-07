#!/usr/bin/env python3

import random
import sys
from time import sleep

STAR_CHAR = "\033[38;2;255;255;0m\033[0m"
BEAM_CHAR = "\033[38;2;0;153;0m󰫤\033[0m"
SPLITTER_CHAR_SELECTION = (
	"\033[38;2;255;153;0m\033[0m",
	"\033[38;2;0;102;255m󰦤\033[0m",
	"\033[38;2;255;0;0m\033[0m",
	"\033[38;2;255;255;102m\033[0m",
)
EMPTY_CHAR = " "


def print_line(line: str, beam: list[bool]) -> None:
	buf = ""

	for lc, bc in zip(line, beam, strict=True):
		if lc == "^":
			buf += random.choice(SPLITTER_CHAR_SELECTION)
		elif bc:
			buf += BEAM_CHAR
		else:
			buf += EMPTY_CHAR

	print(buf)
	sleep(0.1)


def solve(data: str, visualize: bool = False) -> tuple[int, int]:
	assert "^^" not in data, "invalid input: adjacent splitters '^^' found"

	lines = data.splitlines()

	if visualize:
		print("\033c", end="")
		print(data)
		print(f"\033[{len(lines[0]) + 1}F", end="")
		sleep(3)

	cur_beam: list[bool] = [c == "S" for c in lines[0]]
	next_beam: list[bool]

	if visualize:
		print("".join((STAR_CHAR if c == "S" else EMPTY_CHAR) for c in lines[0]))

	p1 = 0

	for line in lines[1:]:
		next_beam = [False] * len(line)

		for i, c in enumerate(line):
			if cur_beam[i]:
				if c == "^":
					next_beam[i - 1] = True
					next_beam[i + 1] = True
					p1 += 1
				else:
					next_beam[i] = True

		if visualize:
			print_line(line, cur_beam)

		cur_beam = next_beam

	return (p1, 0)


test = """\
.......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............
"""
expected = (21, 0)
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
