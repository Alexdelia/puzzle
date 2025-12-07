import sys

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

b: list[int] = []

n = int(input())
c = int(input())
for i in range(n):
	b.append(int(input()))

r: list[int] = [0 for _ in range(n)]

while sum(r) < sum(b):
	for i in range(n - 1, -1, -1):
		if r[i] < b[i]:
			r[i] += 1
		if sum(r) == c:
			for v in sorted(r):
				print(v)
			sys.exit(0)

# Write an answer using print
# To debug: print("Debug messages...", file=sys.stderr, flush=True)

print("IMPOSSIBLE")
