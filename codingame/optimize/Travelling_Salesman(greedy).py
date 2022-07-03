import sys
import math

m = []
u = []

def distance(x1, y1, x2, y2):
    return (math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2))

def closest(curr):
    low_d = 2000
    low_i = 0
    for i in range(len(m)):
        if u[i] == False:
            d = distance(m[curr][0], m[curr][1], m[i][0], m[i][1])
            if d < low_d:
                low_d = d
                low_i = i
    return (low_i)

n = int(input())  # This variables stores how many nodes are given
for i in range(n):
    # x: The x coordinate of the given node
    # y: The y coordinate of the given node
    x, y = [int(j) for j in input().split()]
    m.append((x, y))
    u.append(False)

p = [0]
curr = 0
u[0] = True

while n > 0:
    curr = closest(curr)
    u[curr] = True
    p.append(curr)
    n-=1

# Write an action using print
# To debug: print("Debug messages...", file=sys.stderr, flush=True)

print(*p)
