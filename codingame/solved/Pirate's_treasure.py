import math
import sys
from typing import List

m: List[List[bool]] = []

w = int(input())
h = int(input())

m.append([True] * (w + 2))
for _ in range(h):
	m.append([True] + [c == "1" for c in input().split()] + [True])
	print([int(c) for c in m[-1][1:-1]], file=sys.stderr)
m.append([True] * (w + 2))

for x in range(1, h + 1):
	for y in range(1, w + 1):
		if m[x][y]:
			continue
		c = 0
		for xi in range(-1, 2):
			for yi in range(-1, 2):
				if m[x + xi][y + yi]:
					c += 1
		if c == 8:
			print(y - 1, x - 1)
			sys.exit(0)
