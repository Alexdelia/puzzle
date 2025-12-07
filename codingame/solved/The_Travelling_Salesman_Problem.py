import math
import sys

m = []
u = []


def distance(x1: int, y1: int, x2: int, y2: int) -> float:
	return math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2)


def closest(curr: int) -> int:
	low_d = 2000
	low_i = 0
	for i in range(len(m)):
		if not u[i]:
			d = distance(m[curr][0], m[curr][1], m[i][0], m[i][1])
			if d < low_d:
				low_d = d
				low_i = i
	return low_i


n = int(input())  # This variables stores how many nodes are given
for _ in range(n):
	# x: The x coordinate of the given node
	# y: The y coordinate of the given node
	x, y = [int(j) for j in input().split()]
	m.append((x, y))
	u.append(False)

curr = 0
u[0] = True
t = 0

while n > 0:
	c = closest(curr)
	t += distance(m[curr][0], m[curr][1], m[c][0], m[c][1])
	print(
		f"{curr} -> {c} : {distance(m[curr][0], m[curr][1], m[c][0], m[c][1])}\t({t})",
		file=sys.stderr,
		flush=True,
	)
	curr = c
	u[curr] = True
	n -= 1

print(int(t + 0.5))
