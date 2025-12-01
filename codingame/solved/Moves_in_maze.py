import math
import sys
from typing import List, Tuple

w, h = [int(i) for i in input().split()]
m: List[List[int]] = [[-42 for _ in range(w)] for _ in range(h)]
sx: int = 0
sy: int = 0

for x in range(h):
	for y, c in enumerate(input()):
		if c == "#":
			m[x][y] = -1
		elif c == "S":
			sx, sy = x, y


def periodic(x: int, y: int) -> Tuple[int, int]:
	if x < 0:
		x = h - 1
	elif x >= h:
		x = 0
	if y < 0:
		y = w - 1
	elif y >= w:
		y = 0

	return x, y


def valid(x: int, y: int, v: int) -> bool:
	# if x < 0 or x >= h or y < 0 or y >= w:
	#     return False
	if m[x][y] == -1:
		return False
	elif m[x][y] == -42 or m[x][y] > v:
		return True
	else:
		return False


def flood_fill(x: int, y: int, v: int) -> None:
	q: List[Tuple[int, int, int]] = [(x, y, v)]

	while q:
		x, y, v = q.pop()

		m[x][y] = v

		v += 1
		for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
			nx, ny = periodic(x + dx, y + dy)
			if valid(nx, ny, v):
				q.append((nx, ny, v))


flood_fill(sx, sy, 0)

for x in range(h):
	for y in range(w):
		if m[x][y] == -1:
			print("#", end="")
		elif m[x][y] == -42:
			print(".", end="")
		# print 0..9A..Z
		elif m[x][y] < 10:
			print(m[x][y], end="")
		else:
			print(chr(m[x][y] + 55), end="")
	print()
