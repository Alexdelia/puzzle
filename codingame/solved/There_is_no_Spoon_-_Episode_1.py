import sys
from typing import List

w = int(input())  # the number of cells on the X axis
h = int(input())  # the number of cells on the Y axis
print(w, h, file=sys.stderr)

m: List[List[str]] = []

for x in range(h):
	m.append([])
	for y, c in enumerate(input()):
		m[x].append(c)

for r in m:
	print(r, file=sys.stderr)

for x in range(h):
	for y in range(w):
		if m[x][y] == ".":
			continue
		print(y, x, end=" ")
		for y2 in range(y + 1, w):
			if m[x][y2] == ".":
				continue
			print(y2, x, end=" ")
			break
		else:
			print("-1 -1", end=" ")
		for x2 in range(x + 1, h):
			if m[x2][y] == ".":
				continue
			print(y, x2, end=" ")
			break
		else:
			print("-1 -1", end=" ")
		print()
