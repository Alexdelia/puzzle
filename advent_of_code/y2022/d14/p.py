#!/usr/bin/env python3

import re
from enum import Enum
from os.path import dirname
from time import sleep

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

# DATA = "400,165 -> 568,165 -> 568,150 -> 569,150"

lines = DATA.splitlines()

SIZE = 1000
MIN_X = 481
SIZE_X = 570
SIZE_Y = 167


class Elem(Enum):
	EMPTY = 0
	ROCK = 1
	SAND = 2


m: list[list[Elem]] = [[Elem.EMPTY for _ in range(SIZE)] for _ in range(SIZE)]

max_y = -1
for l in lines:
	s: list[tuple[int, int]] = [
		(int(c.split(",")[0]), int(c.split(",")[1]))
		for c in re.sub(r"->", "", l).split()
	]
	i = 0
	while i < len(s) - 1:
		step_x = [1, -1][s[i][0] > s[i + 1][0]]
		step_y = [1, -1][s[i][1] > s[i + 1][1]]
		for x in range(s[i][0], s[i + 1][0] + step_x, step_x):
			for y in range(s[i][1], s[i + 1][1] + step_y, step_y):
				m[x][y] = Elem.ROCK
				if y > max_y:
					max_y = y
		i += 1


def print_map(m: list[list[Elem]]):
	s = ""

	for y in range(SIZE_Y):
		for x in range(MIN_X, SIZE_X):
			if m[x][y] == Elem.EMPTY:
				s += "  "
			elif m[x][y] == Elem.ROCK:
				s += "\033[48;2;90;70;65m  "
			elif m[x][y] == Elem.SAND:
				s += "\033[33;1m🟡"
			s += "\033[0m"
		s += "\n"

	print(s, flush=True)


def search(m: list[list[Elem]], x: int, y: int) -> tuple[int, int] | bool:
	if y + 1 >= SIZE:
		return False

	if m[x][y + 1] == Elem.EMPTY:
		return (x, y + 1)

	if m[x - 1][y + 1] == Elem.EMPTY:
		return (x - 1, y + 1)
	elif m[x + 1][y + 1] == Elem.EMPTY:
		return (x + 1, y + 1)

	m[x][y] = Elem.SAND
	return True


def sand(m: list[list[Elem]], x: int, y: int) -> bool:
	res = (x, y)

	while isinstance(res, tuple):
		res = search(m, res[0], res[1])

	return res


t = 0
while sand(m, 500, 0):
	if t % 3 == 0:
		print_map(m)
		sleep(0.02)
	t += 1

print_map(m)

p1 = t

for x in range(0, SIZE):
	m[x][max_y + 2] = Elem.ROCK

while m[500][0] != Elem.SAND:
	assert sand(m, 500, 0)
	if t % 20 == 0:
		print_map(m)
		sleep(0.01)
	t += 1

print_map(m)

print(f"part 1:\t{p1}")
print(f"part 2:\t{t}")
