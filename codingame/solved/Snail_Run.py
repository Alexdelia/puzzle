import math
import sys
from typing import List, Tuple

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

class Snail:
    def __init__(self, speed: int):
        self.s: int = speed
        self.w: int = 0

def manathan_distance(p1: Tuple[int, int], p2: Tuple[int, int]) -> int:
    return abs(p1[0] - p2[0]) + abs(p1[1] - p2[1])

s: List[Snail] = []
m: List[List[str]] = []
t: List[Tuple[int, int]] = []

n_s = int(input())
for i in range(n_s):
    s.append(Snail(int(input())))

h = int(input())
w = int(input())
for _ in range(h):
    row = input()
    m.append([c for c in row])
    print(row, file=sys.stderr)

for x in range(h):
    for y in range(w):
        if m[x][y] == '#':
            t.append((x, y))

for x in range(h):
    for y in range(w):
        if m[x][y] >= '0' and m[x][y] <= '9':
            min_dist = 1000000
            for i in t:
                dist = manathan_distance((x, y), i)
                if dist < min_dist:
                    min_dist = dist
            s[int(m[x][y]) - 1].w += int(min_dist)


min_score = s[0].w / s[0].s
min_snail = 0
for i, c in enumerate(s):
    print(f"Snail {i}, s: {c.s}, w: {c.w}, score: {c.w / c.s}", file=sys.stderr)
    score = c.w / c.s
    if score < min_score:
        min_score = score
        min_snail = i

print(min_snail + 1)
