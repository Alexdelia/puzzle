#!/usr/bin/env python3

import re
from os.path import dirname
from typing import List, Optional, Set, Tuple

from aocd import get_data

DAY = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-1]))
YEAR = int(re.sub(r"[^0-9]", "", dirname(__file__).split("/")[-2]))
DATA: str = get_data(day=DAY, year=YEAR)

lines = DATA.splitlines()

for l in lines:
	for i in l:
		print(i, end="")
	print()
print()

m: List[List[int]] = [[0 for _ in range(len(lines[0]))] for _ in range(len(lines))]
ff: List[List[int]] = [[-1 for _ in range(len(lines[0]))] for _ in range(len(lines))]

sx, sy = 0, 0
ex, ey = 0, 0

for x, l in enumerate(lines):
	for y, c in enumerate(l):
		if c == "E":
			ex = x
			ey = y
			m[x][y] = 26
		elif c == "S":
			sx = x
			sy = y
			m[x][y] = 0
		else:
			m[x][y] = ord(c) - ord("a")


def print_heatmap_distance(
	map: List[List[int]], s: Tuple[int, int], e: Tuple[int, int]
):
	t = max([max(r) for r in map])
	for x, r in enumerate(map):
		for y, i in enumerate(r):
			if (x, y) == s:
				print("\033[32;45;1mSS\033[0m", end="")
			elif (x, y) == e:
				print("\033[31;45;1mEE\033[0m", end="")
			else:
				if i == -1:
					print("\033[41m  \033[0m", end="")
				else:
					intensity = int(i / t * 255)
					print(
						f"\033[48;2;{intensity};{255 - intensity};0m  \033[0m", end=""
					)
		print()


def print_heatmap_height(
	map: List[List[int]],
	s: Tuple[int, int],
	e: Tuple[int, int],
	soil: Optional[List[str]] = None,
):
	for x, r in enumerate(map):
		for y in range(len(r)):
			if (x, y) == s:
				print("\033[32;45;1mSS\033[0m", end="")
			elif (x, y) == e:
				print("\033[31;45;1mEE\033[0m", end="")
			else:
				intensity = int(map[x][y] / 26 * 255)
				if soil:
					print(f"\033[38;2;{intensity};{255 - intensity};0m", end="")
					print(soil[x][y] * 2, end="\033[0m")
				else:
					print(
						f"\033[48;2;{intensity};{255 - intensity};0m  ", end="\033[0m"
					)
		print()


print_heatmap_height(m, (sx, sy), (ex, ey))
print()
print_heatmap_height(m, (sx, sy), (ex, ey), lines)
print()


def possible(x: int, y: int) -> List[Tuple[int, int]]:
	ret: List[Tuple[int, int]] = []

	if x > 0 and m[x - 1][y] <= m[x][y] + 1:
		ret.append((x - 1, y))
	if x < len(m) - 1 and m[x + 1][y] <= m[x][y] + 1:
		ret.append((x + 1, y))
	if y > 0 and m[x][y - 1] <= m[x][y] + 1:
		ret.append((x, y - 1))
	if y < len(m[x]) - 1 and m[x][y + 1] <= m[x][y] + 1:
		ret.append((x, y + 1))

	return ret


def flood_fill(x: int, y: int, v: int) -> Set[Tuple[int, int, int]]:
	ret = set()
	ff[x][y] = v
	v += 1

	for p in possible(x, y):
		if ff[p[0]][p[1]] == -1 or ff[p[0]][p[1]] > v:
			ret.add((p[0], p[1], v))

	return ret


def flood(x: int, y: int):
	fl: Set[Tuple[int, int, int]] = set([(x, y, 0)])
	# i = 0

	while fl:
		# if i % 100 == 0:
		#     print_heatmap_distance(ff, (sx, sy), (ex, ey))
		# i += 1

		f = fl.pop()
		fl.update(flood_fill(f[0], f[1], f[2]))


flood(sx, sy)
print_heatmap_distance(ff, (sx, sy), (ex, ey))
print()
p1 = ff[ex][ey]

starts: Set[Tuple[int, int]] = set()
for x, l in enumerate(lines):
	for y, c in enumerate(l):
		if c == "a":
			starts.add((x, y))

min_a: Tuple[int, int, int] = (sx, sy, ff[ex][ey])
for i, s in enumerate(starts):
	print(f"{i + 1} / {len(starts)}", end="\r")
	ff = [[-1 for _ in range(len(lines[0]))] for _ in range(len(lines))]
	flood(s[0], s[1])
	if ff[ex][ey] != -1 and ff[ex][ey] < min_a[2]:
		min_a = (s[0], s[1], ff[ex][ey])

flood(min_a[0], min_a[1])
print_heatmap_distance(ff, (min_a[0], min_a[1]), (ex, ey))
print()

print(f"part 1:\t{p1}")
print(f"part 2:\t{min_a[2]}")
